use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use micro_traffic_sim::pb;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use super::BoxStream;

pub async fn push_session_conflict_zones(
    _sessions: Arc<Mutex<SessionsStorage>>,
    _request: Request<tonic::Streaming<pb::SessionConflictZones>>,
) -> Result<Response<BoxStream<pb::SessionConflictZonesResponse>>, Status> {
    Err(Status::unimplemented("push_session_conflict_zones not implemented"))
}
