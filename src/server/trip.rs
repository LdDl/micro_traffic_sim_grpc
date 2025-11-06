use tonic::{Request, Response, Status};
use micro_traffic_sim::pb;
use super::BoxStream;

pub async fn push_session_trip(
    _request: Request<tonic::Streaming<pb::SessionTrip>>,
) -> Result<Response<BoxStream<pb::SessionTripResponse>>, Status> {
    Err(Status::unimplemented("push_session_trip not implemented"))
}
