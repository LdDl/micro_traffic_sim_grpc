use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tonic::{Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;

/// Shared control + observation handle for one active batch recording.
///
/// Lives in the gRPC layer, NOT the core: a recording's session is removed from
/// `SessionsStorage` and owned by its handler task, so this registry is the only
/// way out-of-band callers (StopRecording / RecordingStatus) can reach a running
/// recording by `session_id`.
#[derive(Default)]
pub struct RecordingHandle {
    /// Last tick written so far (monotonic progress).
    pub progress_tick: AtomicU64,
    /// Total vehicle-ticks (rows) recorded so far.
    pub rows: AtomicU64,
    /// Set by StopRecording; the handler checks it every tick and stops.
    pub cancel: AtomicBool,
}

/// Registry of active batch recordings, keyed by session id. An entry exists
/// exactly while a recording runs; the handler's [`RecordingGuard`] removes it on
/// any exit (completion, stop, error, or panic), so the set is always "currently
/// running recordings".
pub type Recordings = Arc<Mutex<HashMap<Uuid, Arc<RecordingHandle>>>>;

/// Creates an empty recordings registry.
pub fn new_registry() -> Recordings {
    Arc::new(Mutex::new(HashMap::new()))
}

/// RAII deregistration: removes the recording's registry entry on drop, so a
/// finished, aborted, or panicked handler never leaks an entry.
pub struct RecordingGuard {
    recordings: Recordings,
    id: Uuid,
}

impl RecordingGuard {
    pub fn new(recordings: Recordings, id: Uuid) -> Self {
        Self { recordings, id }
    }
}

impl Drop for RecordingGuard {
    fn drop(&mut self) {
        if let Ok(mut reg) = self.recordings.lock() {
            reg.remove(&self.id);
        }
    }
}

fn parse_session_uuid(id: &Option<pb::UuiDv4>) -> Result<Uuid, Status> {
    let value = id
        .as_ref()
        .map(|u| u.value.as_str())
        .ok_or_else(|| Status::invalid_argument("No session ID has been provided"))?;
    Uuid::parse_str(value).map_err(|_| {
        Status::invalid_argument(format!("Session ID should be a UUID v4: '{}'", value))
    })
}

/// Out-of-band: poll the progress/state of a recording by session id.
pub async fn recording_status(
    recordings: Recordings,
    request: Request<pb::RecordingStatusRequest>,
) -> Result<Response<pb::RecordingStatusResponse>, Status> {
    let id = parse_session_uuid(&request.into_inner().session_id)?;
    let resp = {
        let reg = recordings
            .lock()
            .map_err(|_| Status::internal("recordings registry lock poisoned"))?;
        match reg.get(&id) {
            Some(h) => pb::RecordingStatusResponse {
                state: pb::RecordingState::Running as i32,
                current_tick: h.progress_tick.load(Ordering::Relaxed),
                rows: h.rows.load(Ordering::Relaxed),
                cancel_requested: h.cancel.load(Ordering::Relaxed),
            },
            None => pb::RecordingStatusResponse {
                state: pb::RecordingState::NotRunning as i32,
                current_tick: 0,
                rows: 0,
                cancel_requested: false,
            },
        }
    };
    Ok(Response::new(resp))
}

/// Out-of-band: request a running recording to stop (cooperative) by session id.
/// Sets the cancel flag; the handler observes it within one tick, finalizes the
/// current batch, sends RunSummary and ends. Returns `accepted = false` if no
/// recording is running for the session id.
pub async fn stop_recording(
    recordings: Recordings,
    request: Request<pb::StopRecordingRequest>,
) -> Result<Response<pb::StopRecordingResponse>, Status> {
    let id = parse_session_uuid(&request.into_inner().session_id)?;
    let accepted = {
        let reg = recordings
            .lock()
            .map_err(|_| Status::internal("recordings registry lock poisoned"))?;
        match reg.get(&id) {
            Some(h) => {
                h.cancel.store(true, Ordering::Relaxed);
                true
            }
            None => false,
        }
    };
    Ok(Response::new(pb::StopRecordingResponse { accepted }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_req(id: Uuid) -> Request<pb::RecordingStatusRequest> {
        Request::new(pb::RecordingStatusRequest {
            session_id: Some(pb::UuiDv4 {
                value: id.to_string(),
            }),
        })
    }
    fn stop_req(id: Uuid) -> Request<pb::StopRecordingRequest> {
        Request::new(pb::StopRecordingRequest {
            session_id: Some(pb::UuiDv4 {
                value: id.to_string(),
            }),
        })
    }

    #[tokio::test]
    async fn status_and_stop_via_registry() {
        let reg = new_registry();
        let id = Uuid::new_v4();

        // Nothing running yet.
        let r = recording_status(reg.clone(), status_req(id))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.state, pb::RecordingState::NotRunning as i32);
        let s = stop_recording(reg.clone(), stop_req(id))
            .await
            .unwrap()
            .into_inner();
        assert!(!s.accepted, "stop on a non-running recording is a no-op");

        // Register a running recording with some progress.
        let handle = Arc::new(RecordingHandle::default());
        handle.progress_tick.store(42, Ordering::Relaxed);
        handle.rows.store(100, Ordering::Relaxed);
        reg.lock().unwrap().insert(id, handle.clone());

        // Status reports it.
        let r = recording_status(reg.clone(), status_req(id))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.state, pb::RecordingState::Running as i32);
        assert_eq!(r.current_tick, 42);
        assert_eq!(r.rows, 100);
        assert!(!r.cancel_requested);

        // Stop sets the cancel flag, which the handler would observe.
        let s = stop_recording(reg.clone(), stop_req(id))
            .await
            .unwrap()
            .into_inner();
        assert!(s.accepted);
        assert!(handle.cancel.load(Ordering::Relaxed));
        let r = recording_status(reg.clone(), status_req(id))
            .await
            .unwrap()
            .into_inner();
        assert!(r.cancel_requested);
    }
}
