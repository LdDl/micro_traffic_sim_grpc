use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;
use micro_traffic_sim_core::agents_types::AgentType;
use micro_traffic_sim_core::behaviour::BehaviourType;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use micro_traffic_sim_core::trips::trip::{Trip, TripType};

use super::BoxStream;

/// Converts proto TripType (i32) to computational core TripType
fn proto_trip_type_to_core(trip_type: i32) -> TripType {
    match trip_type {
        1 => TripType::Constant,
        2 => TripType::Random,
        _ => TripType::Undefined,
    }
}

/// Converts proto BehaviourType (i32) to computational core BehaviourType
fn proto_behaviour_type_to_core(behaviour_type: i32) -> BehaviourType {
    match behaviour_type {
        1 => BehaviourType::Block,
        2 => BehaviourType::Aggressive,
        3 => BehaviourType::Cooperative,
        4 => BehaviourType::LimitSpeedByTrip,
        _ => BehaviourType::Undefined,
    }
}

/// Converts proto AgentType (i32) to computational core AgentType
fn proto_agent_type_to_core(agent_type: i32) -> AgentType {
    match agent_type {
        1 => AgentType::Car,
        2 => AgentType::Bus,
        3 => AgentType::Taxi,
        4 => AgentType::Pedestrian,
        _ => AgentType::Undefined,
    }
}

pub async fn push_session_trip(
    sessions: Arc<Mutex<SessionsStorage>>,
    request: Request<tonic::Streaming<pb::SessionTrip>>,
) -> Result<Response<BoxStream<pb::SessionTripResponse>>, Status> {
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

            // Validate data size
            if req.data.len() > 10000 {
                let _ = tx
                    .send(Err(Status::invalid_argument(format!(
                        "Max amount on data entities is 10000, but provided is {}",
                        req.data.len()
                    ))))
                    .await;
                return;
            }

            if req.data.is_empty() {
                let _ = tx
                    .send(Err(Status::invalid_argument("No data")))
                    .await;
                return;
            }

            // Get session and add trips (use block scope to ensure lock is dropped before await)
            let add_result = {
                let mut sessions_guard = sessions.lock().unwrap();
                sessions_guard.with_session_mut(&session_uuid, |session| {
                    // Convert proto trips to core trips and add them
                    for trip_data in &req.data {
                        let trip_type = proto_trip_type_to_core(trip_data.trip_type);
                        let behaviour_type = proto_behaviour_type_to_core(trip_data.behaviour_type);
                        let agent_type = proto_agent_type_to_core(trip_data.agent_type);

                        // Convert transits vector
                        let transits: Vec<i64> = trip_data.transits.clone();

                        // Build trip using the builder pattern
                        let mut trip_builder = Trip::new(trip_data.from_node, trip_data.to_node, trip_type)
                            .with_id(trip_data.id)
                            .with_initial_speed(trip_data.initial_speed as i32)
                            .with_probability(trip_data.probability)
                            .with_allowed_agent_type(agent_type)
                            .with_allowed_behaviour_type(behaviour_type)
                            .with_time(trip_data.time as i32)
                            .with_start_time(trip_data.start_time as i32)
                            .with_end_time(trip_data.end_time as i32);

                        // Set transits if any
                        if !transits.is_empty() {
                            trip_builder = trip_builder.with_transits_cells(transits, trip_data.relax_time as i32);
                        }

                        let trip = trip_builder.build();
                        session.add_trip(trip);
                    }
                })
            };

            if add_result.is_none() {
                let _ = tx
                    .send(Err(Status::not_found(format!(
                        "Not found session ID: '{}'",
                        session_id
                    ))))
                    .await;
                return;
            }

            // Send OK response
            let resp = pb::SessionTripResponse {
                code: Code::Ok as u32,
                text: "OK".to_string(),
            };
            if tx.send(Ok(resp)).await.is_err() {
                return;
            }
        }
    });

    let out: BoxStream<pb::SessionTripResponse> = Box::pin(ReceiverStream::new(rx));
    Ok(Response::new(out))
}
