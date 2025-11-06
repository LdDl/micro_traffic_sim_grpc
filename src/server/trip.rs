use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use micro_traffic_sim::pb;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use super::BoxStream;

pub async fn push_session_trip(
    _sessions: Arc<Mutex<SessionsStorage>>,
    _request: Request<tonic::Streaming<pb::SessionTrip>>,
) -> Result<Response<BoxStream<pb::SessionTripResponse>>, Status> {
    Err(Status::unimplemented("push_session_trip not implemented"))
}
