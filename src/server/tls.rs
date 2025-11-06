use tonic::{Request, Response, Status};
use micro_traffic_sim::pb;
use super::BoxStream;

pub async fn push_session_tls(
    _request: Request<tonic::Streaming<pb::SessionTls>>,
) -> Result<Response<BoxStream<pb::SessionTlsResponse>>, Status> {
    Err(Status::unimplemented("push_session_tls not implemented"))
}
