use tonic::{Request, Response, Status};
use micro_traffic_sim::pb;

pub async fn new_session(
    _request: Request<pb::SessionReq>,
) -> Result<Response<pb::NewSessionResponse>, Status> {
    Err(Status::unimplemented("new_session not implemented"))
}

pub async fn info_session(
    _request: Request<pb::UuiDv4>,
) -> Result<Response<pb::InfoSessionResponse>, Status> {
    Err(Status::unimplemented("info_session not implemented"))
}
