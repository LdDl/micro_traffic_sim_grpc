use std::sync::{Arc, Mutex};
use std::time::Duration;
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;
use micro_traffic_sim_core::geom::SRID;
use micro_traffic_sim_core::simulation::session::Session;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use micro_traffic_sim_core::verbose::VerboseLevel;

pub async fn new_session(
    sessions: Arc<Mutex<SessionsStorage>>,
    session_verbose: VerboseLevel,
    request: Request<pb::SessionReq>,
) -> Result<Response<pb::NewSessionResponse>, Status> {
    let srid = match request.into_inner().srid {
        4326 => Some(SRID::WGS84),
        0 => Some(SRID::Euclidean),
        _ => None, // defaults inside Session::default
    };

    let mut session = Session::default(srid);
    session.set_verbose_level(session_verbose);
    let sid = session.get_id();

    let ttl = Some(Duration::from_secs(4 * 60));
    let mut guard = sessions.lock().map_err(|_| Status::internal("storage poisoned"))?;
    let _ = guard.register_session(sid, session, ttl);
    drop(guard);

    let resp = pb::NewSessionResponse {
        code: Code::Ok as u32,
        text: Code::Ok.to_string(),
        id: Some(pb::UuiDv4 { value: sid.to_string() }),
    };
    Ok(Response::new(resp))
}

pub async fn info_session(
    sessions: Arc<Mutex<SessionsStorage>>,
    request: Request<pb::UuiDv4>,
) -> Result<Response<pb::InfoSessionResponse>, Status> {
    let id = request.into_inner().value;
    let sid = Uuid::parse_str(&id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

    let mut guard = sessions.lock().map_err(|_| Status::internal("storage poisoned"))?;
    // with_session_mut extends TTL; we just check presence
    let found = guard.with_session_mut(&sid, |sess| sess.get_id()).is_some();
    drop(guard);

    if !found {
        let resp = pb::InfoSessionResponse {
            code: Code::NotFound as u32,
            text: Code::NotFound.to_string(),
            data: None,
        };
        return Ok(Response::new(resp));
    }

    let resp = pb::InfoSessionResponse {
        code: Code::Ok as u32,
        text: Code::Ok.to_string(),
        data: Some(pb::Session { id: Some(pb::UuiDv4 { value: sid.to_string() }) }),
    };
    Ok(Response::new(resp))
}
