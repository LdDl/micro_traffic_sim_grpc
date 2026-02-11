use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use futures_core::Stream;
use tonic::{transport::Server, Request, Response, Status};

use micro_traffic_sim::pb;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use micro_traffic_sim_core::verbose::VerboseLevel;

// Submodules with per-RPC handlers (keep logic out of this file)
mod sessions;
mod grid;
mod trip;
mod step;
mod tls;
mod conflict_zones;

// Shared stream type alias for bidirectional streaming
pub(super) type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

struct SimService {
    sessions: Arc<Mutex<SessionsStorage>>,
    session_verbose: VerboseLevel,
}

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
        sessions::new_session(self.sessions.clone(), self.session_verbose, request).await
    }

    async fn info_session(
        &self,
        request: Request<pb::UuiDv4>,
    ) -> Result<Response<pb::InfoSessionResponse>, Status> {
        sessions::info_session(self.sessions.clone(), request).await
    }

    async fn push_session_grid(
        &self,
        request: Request<tonic::Streaming<pb::SessionGrid>>,
    ) -> Result<Response<Self::PushSessionGridStream>, Status> {
        grid::push_session_grid(self.sessions.clone(), request).await
    }

    async fn push_session_trip(
        &self,
        request: Request<tonic::Streaming<pb::SessionTrip>>,
    ) -> Result<Response<Self::PushSessionTripStream>, Status> {
        trip::push_session_trip(self.sessions.clone(), request).await
    }

    async fn simulation_step_session(
        &self,
        request: Request<tonic::Streaming<pb::SessionStep>>,
    ) -> Result<Response<Self::SimulationStepSessionStream>, Status> {
        step::simulation_step_session(self.sessions.clone(), request).await
    }

    async fn push_session_tls(
        &self,
        request: Request<tonic::Streaming<pb::SessionTls>>,
    ) -> Result<Response<Self::PushSessionTLSStream>, Status> {
        tls::push_session_tls(self.sessions.clone(), request).await
    }

    async fn push_session_conflict_zones(
        &self,
        request: Request<tonic::Streaming<pb::SessionConflictZones>>,
    ) -> Result<Response<Self::PushSessionConflictZonesStream>, Status> {
        conflict_zones::push_session_conflict_zones(self.sessions.clone(), request).await
    }
}

fn spawn_purge_task(sessions: Arc<Mutex<SessionsStorage>>) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(30); // keep in sync with with_purge_every
        loop {
            sleep(interval).await;
            if let Ok(mut guard) = sessions.lock() {
                guard.purge_expired();
            }
        }
    });
}

pub async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    let default_addr = "0.0.0.0:50051";
    let addr: SocketAddr = std::env::var("MT_SIM_ADDR")
        .unwrap_or_else(|_| default_addr.to_string())
        .parse()?;
    // Parse verbose level helper
    let parse_verbose = |val: &str| -> VerboseLevel {
        match val {
            "0" => VerboseLevel::None,
            "2" => VerboseLevel::Additional,
            _ => VerboseLevel::Main,
        }
    };
    // MT_SIM_VERBOSE: per-session simulation logging (steps, conflicts, movement). Default: 0 (None)
    let sim_verbose = parse_verbose(
        &std::env::var("MT_SIM_VERBOSE").unwrap_or_else(|_| "0".to_string()),
    );
    // MT_SIM_SERVICE_VERBOSE: storage-level logging (session create/expire). Default: 1 (Main)
    let storage_verbose = parse_verbose(
        &std::env::var("MT_SIM_SERVICE_VERBOSE").unwrap_or_else(|_| "1".to_string()),
    );
    // Configure a shared SessionsStorage for the server
    let store = SessionsStorage::new()
        .with_session_exp_time(Duration::from_secs(4 * 60))
        .with_purge_every(Duration::from_secs(30))
        .with_storage_verbose(storage_verbose);
    let sessions = Arc::new(Mutex::new(store));
    spawn_purge_task(sessions.clone());

    let svc = pb::service_server::ServiceServer::new(SimService {
        sessions: sessions.clone(),
        session_verbose: sim_verbose,
    });

    println!("Starting micro_traffic_sim gRPC server on {}", addr);
    Server::builder()
        .add_service(svc)
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            println!("\nShutting down gRPC server...");
        })
        .await?;
    Ok(())
}

pub fn run_blocking() {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    if let Err(e) = rt.block_on(main_async()) {
        eprintln!("Server failed: {e}");
    }
}
