//! # micro_traffic_sim
//!
//! gRPC interface for microscopic traffic simulation via cellular automata.
//!
//! This crate provides a [tonic]-based gRPC client and server for interacting with
//! the [`micro_traffic_sim_core`] simulation engine. It allows you to:
//!
//! - Create simulation sessions
//! - Define road networks as cellular grids / graphs
//! - Configure traffic lights with signal phases
//! - Set up conflict zones for unregulated intersections or where traffic lights have conflicting green phases
//! - Setup vehicle generators via trips technique
//! - Step through the simulation and observe vehicle/traffic light states
//!
//! ## Architecture
//!
//! The simulation core ([`micro_traffic_sim_core`]) implements the cellular automaton
//! model for traffic flow. This crate wraps it with a gRPC API defined in Protocol Buffers,
//! enabling language-agnostic access from Go, Python, or any gRPC-compatible client.
//!
//! ## Quick Start (Client)
//!
//! ```rust,no_run
//! use micro_traffic_sim::pb::service_client::ServiceClient;
//! use micro_traffic_sim::pb::{SessionReq, UuiDv4, SessionGrid, Cell, Point};
//! use tonic::transport::Channel;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to the gRPC server
//!     let channel = Channel::from_static("http://127.0.0.1:50051")
//!         .connect()
//!         .await?;
//!     let mut client = ServiceClient::new(channel);
//!
//!     // Create a new session (SRID 0 = Euclidean coordinates)
//!     let response = client.new_session(SessionReq { srid: 0 }).await?;
//!     let session_id = response.into_inner().id.unwrap().value;
//!     println!("Session created: {}", session_id);
//!
//!     // Now push grid cells, trips, traffic lights, and run simulation steps...
//!     Ok(())
//! }
//! ```
//!
//! For a complete working example, see [`examples/rust_client`](https://github.com/LdDl/micro_traffic_sim_grpc/tree/master/examples/rust_client).
//!
//! ## Running the Server
//!
//! The server binary is included when built with the `server` feature:
//!
//! ```sh
//! cargo run --features server --bin micro_traffic_sim
//! ```
//!
//! ## Protocol Buffers
//!
//! All types are generated from `.proto` files and exposed under the [`pb`] module:
//!
//! - [`pb::service_client::ServiceClient`] - gRPC client stub
//! - [`pb::service_server::ServiceServer`] - gRPC server trait (with `server` feature)
//! - [`pb::Cell`] - Road network cell
//! - [`pb::Trip`] - Vehicle trip/generator configuration
//! - [`pb::TrafficLight`] - Traffic light with signal groups
//! - [`pb::ConflictZone`] - Priority rules for unregulated intersections
//! - [`pb::SessionStep`] / [`pb::SessionStepResponse`] - Simulation step request/response
//! - [`pb::VehicleState`] - Vehicle position and state per timestep
//!
//! ## Related Crates
//!
//! - [`micro_traffic_sim_core`] - The computation engine (cellular automaton implementation)
//!
//! ## Clients in Other Languages
//!
//! - **Go**: [clients/go](https://github.com/LdDl/micro_traffic_sim_grpc/tree/master/clients/go) ([pkg.go.dev](https://pkg.go.dev/github.com/LdDl/micro_traffic_sim_grpc/clients/go))
//! - **Python**: [clients/python](https://github.com/LdDl/micro_traffic_sim_grpc/tree/master/clients/python) ([PyPI](https://pypi.org/project/micro-traffic-sim/))
//!
//! [`micro_traffic_sim_core`]: https://docs.rs/micro_traffic_sim_core/latest/micro_traffic_sim_core/
//! [tonic]: https://docs.rs/tonic/latest/tonic/

/// Generated Protocol Buffer types and gRPC service definitions.
///
/// This module contains all types generated from the `.proto` files:
///
/// - **Session management**: [`SessionReq`], [`NewSessionResponse`], [`InfoSessionResponse`]
/// - **Grid/Cells**: [`Cell`], [`Point`], [`SessionGrid`], [`ZoneType`]
/// - **Trips**: [`Trip`], [`SessionTrip`], [`TripType`], [`AgentType`], [`BehaviourType`]
/// - **Traffic Lights**: [`TrafficLight`], [`Group`], [`GroupType`], [`SessionTls`]
/// - **Conflict Zones**: [`ConflictZone`], [`SessionConflictZones`], [`ConflictWinnerType`]
/// - **Simulation**: [`SessionStep`], [`SessionStepResponse`], [`VehicleState`], [`TlsState`]
/// - **gRPC Client**: [`service_client::ServiceClient`]
/// - **gRPC Server**: [`service_server::ServiceServer`] (with `server` feature)
///
/// [`SessionReq`]: SessionReq
/// [`NewSessionResponse`]: NewSessionResponse
/// [`InfoSessionResponse`]: InfoSessionResponse
/// [`Cell`]: Cell
/// [`Point`]: Point
/// [`SessionGrid`]: SessionGrid
/// [`ZoneType`]: ZoneType
/// [`Trip`]: Trip
/// [`SessionTrip`]: SessionTrip
/// [`TripType`]: TripType
/// [`AgentType`]: AgentType
/// [`BehaviourType`]: BehaviourType
/// [`TrafficLight`]: TrafficLight
/// [`Group`]: Group
/// [`GroupType`]: GroupType
/// [`SessionTls`]: SessionTls
/// [`ConflictZone`]: ConflictZone
/// [`SessionConflictZones`]: SessionConflictZones
/// [`ConflictWinnerType`]: ConflictWinnerType
/// [`SessionStep`]: SessionStep
/// [`SessionStepResponse`]: SessionStepResponse
/// [`VehicleState`]: VehicleState
/// [`TlsState`]: TlsState
/// [`service_client::ServiceClient`]: service_client::ServiceClient
/// [`service_server::ServiceServer`]: service_server::ServiceServer
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/micro_traffic_sim.rs"));
}

// Re-export all generated types at crate root for convenience.
pub use pb::*;
