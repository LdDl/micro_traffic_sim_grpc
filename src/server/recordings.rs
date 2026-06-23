use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

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
