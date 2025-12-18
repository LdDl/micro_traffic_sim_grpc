use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;
use micro_traffic_sim_core::conflict_zones::{
    ConflictEdge, ConflictWinnerType, ConflictZone, ConflictZoneType,
};
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;

use super::BoxStream;

/// Converts proto ConflictWinnerType (i32) to computational core ConflictWinnerType
fn proto_winner_type_to_core(winner_type: i32) -> ConflictWinnerType {
    match winner_type {
        1 => ConflictWinnerType::Equal,
        2 => ConflictWinnerType::First,
        3 => ConflictWinnerType::Second,
        _ => ConflictWinnerType::Undefined,
    }
}

/// Converts proto ConflictZoneType (i32) to computational core ConflictZoneType
fn proto_zone_type_to_core(_zone_type: i32) -> ConflictZoneType {
    // Currently only Undefined is implemented in core
    ConflictZoneType::Undefined
}

pub async fn push_session_conflict_zones(
    sessions: Arc<Mutex<SessionsStorage>>,
    request: Request<tonic::Streaming<pb::SessionConflictZones>>,
) -> Result<Response<BoxStream<pb::SessionConflictZonesResponse>>, Status> {
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

            // Get session
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

            // Convert proto conflict zones to core conflict zones and add them
            for cz_data in &req.data {
                let first_edge = ConflictEdge {
                    source: cz_data.source_x,
                    target: cz_data.target_x,
                };
                let second_edge = ConflictEdge {
                    source: cz_data.source_y,
                    target: cz_data.target_y,
                };

                let winner_type = proto_winner_type_to_core(cz_data.conflict_winner);
                let zone_type = proto_zone_type_to_core(cz_data.conflict_type);

                let conflict_zone = ConflictZone::new(cz_data.id as i32, first_edge, second_edge)
                    .with_winner_type(winner_type)
                    .with_zone_type(zone_type)
                    .build();

                session.add_conflict_zone(conflict_zone);
            }

            // Drop the lock before sending response
            drop(sessions_guard);

            // Send OK response
            let resp = pb::SessionConflictZonesResponse {
                code: Code::Ok as u32,
                text: "OK".to_string(),
            };
            if tx.send(Ok(resp)).await.is_err() {
                return;
            }
        }
    });

    let out: BoxStream<pb::SessionConflictZonesResponse> = Box::pin(ReceiverStream::new(rx));
    Ok(Response::new(out))
}
