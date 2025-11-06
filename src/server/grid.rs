use tonic::{Request, Response, Status};
use micro_traffic_sim::pb;
use super::BoxStream;

pub async fn push_session_grid(
    _request: Request<tonic::Streaming<pb::SessionGrid>>,
) -> Result<Response<BoxStream<pb::SessionGridResponse>>, Status> {
    Err(Status::unimplemented("push_session_grid not implemented"))
}
