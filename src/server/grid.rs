use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;
use micro_traffic_sim_core::geom::new_point;
use micro_traffic_sim_core::grid::cell::Cell;
use micro_traffic_sim_core::grid::zones::ZoneType;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;

use super::BoxStream;

/// Converts proto ZoneType to computational core ZoneType
fn proto_zone_to_core(zone: i32) -> ZoneType {
    match zone {
        1 => ZoneType::Birth,
        2 => ZoneType::Death,
        3 => ZoneType::Coordination,
        4 => ZoneType::Common,
        5 => ZoneType::Isolated,
        6 => ZoneType::LaneForBus,
        7 => ZoneType::Transit,
        8 => ZoneType::Crosswalk,
        _ => ZoneType::Undefined,
    }
}

pub async fn push_session_grid(
    sessions: Arc<Mutex<SessionsStorage>>,
    request: Request<tonic::Streaming<pb::SessionGrid>>,
) -> Result<Response<BoxStream<pb::SessionGridResponse>>, Status> {
    let mut stream = request.into_inner();
    let (tx, rx) = mpsc::channel(16);

    tokio::spawn(async move {
        while let Ok(Some(req)) = stream.message().await {
            // Validate session_id
            let session_id = match req.session_id {
                Some(id) => id.value,
                None => {
                    let _ = tx
                        .send(Err(Status::invalid_argument("No session ID provided")))
                        .await;
                    continue;
                }
            };

            let sid = match Uuid::parse_str(&session_id) {
                Ok(u) => u,
                Err(_) => {
                    let _ = tx
                        .send(Err(Status::invalid_argument("Invalid UUID format")))
                        .await;
                    continue;
                }
            };

            // Validate data size
            if req.data.len() > 10000 {
                let _ = tx
                    .send(Err(Status::invalid_argument(format!(
                        "Max amount of data entities is 10000, but provided is {}",
                        req.data.len()
                    ))))
                    .await;
                continue;
            }

            if req.data.is_empty() {
                let _ = tx
                    .send(Err(Status::invalid_argument("No data provided")))
                    .await;
                continue;
            }

            // Get session and SRID
            let srid_result = sessions
                .lock()
                .ok()
                .and_then(|mut guard| guard.with_session_mut(&sid, |session| session.get_world_srid()));

            let srid = match srid_result {
                Some(s) => s,
                None => {
                    // Either lock poisoned or session not found - try to distinguish
                    if sessions.lock().is_err() {
                        let _ = tx.send(Err(Status::internal("Storage poisoned"))).await;
                    } else {
                        let _ = tx
                            .send(Err(Status::not_found(format!(
                                "Session not found: {}",
                                session_id
                            ))))
                            .await;
                    }
                    continue;
                }
            };

            // Convert proto cells to core cells
            let cells_data: Vec<Cell> = req
                .data
                .iter()
                .map(|c| {
                    let (x, y) = c.geom.as_ref().map_or((0.0, 0.0), |p| (p.x, p.y));
                    Cell::new(c.id)
                        .with_point(new_point(x, y, Some(srid)))
                        .with_zone_type(proto_zone_to_core(c.zone_type))
                        .with_speed_limit(c.speed_limit as i32)
                        .with_left_node(c.left_node)
                        .with_forward_node(c.forward_node)
                        .with_right_node(c.right_node)
                        .with_meso_link(c.meso_link_id)
                        .build()
                })
                .collect();

            // Add cells to session
            let add_result = sessions
                .lock()
                .ok()
                .and_then(|mut guard| {
                    guard.with_session_mut(&sid, |session| {
                        session.add_cells(cells_data);
                    })
                });

            if add_result.is_none() {
                // Session disappeared between SRID fetch and add - rare but possible
                let _ = tx
                    .send(Err(Status::not_found(format!(
                        "Session not found: {}",
                        session_id
                    ))))
                    .await;
                continue;
            }

            let resp = pb::SessionGridResponse {
                code: Code::Ok as u32,
                text: Code::Ok.to_string(),
            };

            if tx.send(Ok(resp)).await.is_err() {
                break;
            }
        }
    });

    let out: BoxStream<pb::SessionGridResponse> = Box::pin(ReceiverStream::new(rx));
    Ok(Response::new(out))
}
