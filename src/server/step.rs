use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use micro_traffic_sim::pb;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use super::BoxStream;

pub async fn simulation_step_session(
    _sessions: Arc<Mutex<SessionsStorage>>,
    _request: Request<tonic::Streaming<pb::SessionStep>>,
) -> Result<Response<BoxStream<pb::SessionStepResponse>>, Status> {
    Err(Status::unimplemented("simulation_step_session not implemented"))
}
