use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;
use micro_traffic_sim_core::agents_types::AgentType;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;

use super::BoxStream;

/// Converts core AgentType to proto AgentType (i32)
fn core_agent_type_to_proto(agent_type: AgentType) -> i32 {
    match agent_type {
        AgentType::Undefined => 0,
        AgentType::Car => 1,
        AgentType::Bus => 2,
        AgentType::Taxi => 3,
        AgentType::Pedestrian => 4,
    }
}

pub async fn simulation_step_session(
    sessions: Arc<Mutex<SessionsStorage>>,
    request: Request<tonic::Streaming<pb::SessionStep>>,
) -> Result<Response<BoxStream<pb::SessionStepResponse>>, Status> {
    let mut stream = request.into_inner();
    let (tx, rx) = mpsc::channel(16);

    tokio::spawn(async move {
        while let Ok(Some(req)) = stream.message().await {
            // Validate session_id presence
            let session_id = match &req.session_id {
                Some(id) => &id.value,
                None => {
                    let _ = tx
                        .send(Err(Status::invalid_argument("No session ID has been provided")))
                        .await;
                    return;
                }
            };

            // Parse UUID
            let session_uuid = match Uuid::parse_str(session_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    let _ = tx
                        .send(Err(Status::invalid_argument(format!(
                            "Session ID should be of type UUID v4: '{}'",
                            session_id
                        ))))
                        .await;
                    return;
                }
            };

            // Get session and run step
            let mut sessions_guard = sessions.lock().unwrap();
            let session = match sessions_guard.get_session_mut(&session_uuid) {
                Some(s) => s,
                None => {
                    let _ = tx
                        .send(Err(Status::not_found(format!(
                            "Not found session ID: '{}'",
                            session_id
                        ))))
                        .await;
                    return;
                }
            };

            // Run simulation step
            let dump = match session.step() {
                Ok(state) => state,
                Err(e) => {
                    let _ = tx
                        .send(Err(Status::aborted(e.to_string())))
                        .await;
                    return;
                }
            };

            // Drop the lock before building response
            drop(sessions_guard);

            // Convert vehicle states
            let vehicle_data: Vec<pb::VehicleState> = dump
                .vehicles
                .iter()
                .map(|v| {
                    let intermediate_cells: Vec<i64> = v.last_intermediate_cells.clone();
                    let tail_cells: Vec<i64> = v.tail_cells.clone();

                    pb::VehicleState {
                        vehicle_id: v.id,
                        vehicle_type: core_agent_type_to_proto(v.vehicle_type),
                        speed: v.last_speed as i64,
                        bearing: v.last_angle,
                        cell: v.last_cell,
                        intermediate_cells,
                        point: Some(pb::Point {
                            x: v.last_point[0],
                            y: v.last_point[1],
                        }),
                        travel_time: v.travel_time,
                        trip_id: v.trip_id,
                        tail_cells,
                    }
                })
                .collect();

            // Convert TLS states
            let tls_data: Vec<pb::TlsState> = dump
                .tls
                .iter()
                .map(|(tl_id, groups)| {
                    let groups_proto: Vec<pb::TlGroup> = groups
                        .iter()
                        .map(|g| pb::TlGroup {
                            id: g.group_id,
                            signal: g.last_signal.to_string(),
                        })
                        .collect();

                    pb::TlsState {
                        id: *tl_id,
                        groups: groups_proto,
                    }
                })
                .collect();

            // Send response
            let resp = pb::SessionStepResponse {
                code: Code::Ok as u32,
                text: "OK".to_string(),
                timestamp: dump.timestamp as i64,
                vehicle_data,
                tls_data,
            };

            if tx.send(Ok(resp)).await.is_err() {
                return;
            }
        }
    });

    let out: BoxStream<pb::SessionStepResponse> = Box::pin(ReceiverStream::new(rx));
    Ok(Response::new(out))
}
