use std::net::SocketAddr;
use std::pin::Pin;
use futures_core::Stream;
use tonic::{transport::Server, Request, Response, Status};

use micro_traffic_sim::pb;

// Submodules with per-RPC handlers (keep logic out of this file)
mod sessions;
mod grid;
mod trip;
mod step;
mod tls;
mod conflict_zones;

// Shared stream type alias for bidirectional streaming
pub(super) type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

#[derive(Debug, Default)]
struct SimService;

#[tonic::async_trait]
impl pb::service_server::Service for SimService {
    type PushSessionGridStream = BoxStream<pb::SessionGridResponse>;
    type PushSessionTripStream = BoxStream<pb::SessionTripResponse>;
    type SimulationStepSessionStream = BoxStream<pb::SessionStepResponse>;
    type PushSessionTLSStream = BoxStream<pb::SessionTlsResponse>;
    type PushSessionConflictZonesStream = BoxStream<pb::SessionConflictZonesResponse>;

    async fn new_session(
        &self,
        request: Request<pb::SessionReq>,
    ) -> Result<Response<pb::NewSessionResponse>, Status> {
        sessions::new_session(request).await
    }

    async fn info_session(
        &self,
        request: Request<pb::UuiDv4>,
    ) -> Result<Response<pb::InfoSessionResponse>, Status> {
        sessions::info_session(request).await
    }

    async fn push_session_grid(
        &self,
        request: Request<tonic::Streaming<pb::SessionGrid>>,
    ) -> Result<Response<Self::PushSessionGridStream>, Status> {
        grid::push_session_grid(request).await
    }

    async fn push_session_trip(
        &self,
        request: Request<tonic::Streaming<pb::SessionTrip>>,
    ) -> Result<Response<Self::PushSessionTripStream>, Status> {
        trip::push_session_trip(request).await
    }

    async fn simulation_step_session(
        &self,
        request: Request<tonic::Streaming<pb::SessionStep>>,
    ) -> Result<Response<Self::SimulationStepSessionStream>, Status> {
        step::simulation_step_session(request).await
    }

    async fn push_session_tls(
        &self,
        request: Request<tonic::Streaming<pb::SessionTls>>,
    ) -> Result<Response<Self::PushSessionTLSStream>, Status> {
        tls::push_session_tls(request).await
    }

    async fn push_session_conflict_zones(
        &self,
        request: Request<tonic::Streaming<pb::SessionConflictZones>>,
    ) -> Result<Response<Self::PushSessionConflictZonesStream>, Status> {
        conflict_zones::push_session_conflict_zones(request).await
    }
}

pub async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "0.0.0.0:50051".parse()?;
    let svc = pb::service_server::ServiceServer::new(SimService::default());

    println!("Starting micro_traffic_sim gRPC server on {}", addr);
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

pub fn run_blocking() {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    if let Err(e) = rt.block_on(main_async()) {
        eprintln!("Server failed: {e}");
    }
}
