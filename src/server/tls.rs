use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;
use micro_traffic_sim_core::geom::new_point;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use micro_traffic_sim_core::traffic_lights::groups::TrafficLightGroup;
use micro_traffic_sim_core::traffic_lights::lights::TrafficLight;
use micro_traffic_sim_core::traffic_lights::signals::SignalType;

use super::BoxStream;

pub async fn push_session_tls(
    sessions: Arc<Mutex<SessionsStorage>>,
    request: Request<tonic::Streaming<pb::SessionTls>>,
) -> Result<Response<BoxStream<pb::SessionTlsResponse>>, Status> {
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

            // Handle empty data with warning (like Go does)
            if req.data.is_empty() {
                let resp = pb::SessionTlsResponse {
                    code: Code::Ok as u32,
                    text: "[WARNING] Status: OK. No data".to_string(),
                };
                let _ = tx.send(Ok(resp)).await;
                continue;
            }

            // Pre-validate all signals first (before acquiring session)
            // This captures any signal parsing errors early
            let mut parsed_signals: Vec<Vec<Vec<SignalType>>> = Vec::with_capacity(req.data.len());
            for tl_data in &req.data {
                let mut tl_signals = Vec::with_capacity(tl_data.groups.len());
                for group_data in &tl_data.groups {
                    let mut signals: Vec<SignalType> = Vec::with_capacity(group_data.signals.len());
                    for (sig_idx, sig_str) in group_data.signals.iter().enumerate() {
                        match SignalType::from_str(sig_str) {
                            Ok(signal) => signals.push(signal),
                            Err(_) => {
                                let _ = tx
                                    .send(Err(Status::invalid_argument(format!(
                                        "Signal type '{}' not supported (group {}, signal index {})",
                                        sig_str, group_data.id, sig_idx
                                    ))))
                                    .await;
                                return;
                            }
                        }
                    }
                    tl_signals.push(signals);
                }
                parsed_signals.push(tl_signals);
            }

            // Get session and add traffic lights (use block scope to ensure lock is dropped before await)
            let add_result = {
                let mut sessions_guard = sessions.lock().unwrap();
                sessions_guard.with_session_mut(&session_uuid, |session| {
                    let srid = session.get_world_srid();

                    // Convert proto traffic lights to core traffic lights
                    for (tl_idx, tl_data) in req.data.iter().enumerate() {
                        // Convert times
                        let times: Vec<i32> = tl_data.times.iter().map(|t| *t as i32).collect();

                        // Convert groups
                        let mut groups: Vec<TrafficLightGroup> = Vec::with_capacity(tl_data.groups.len());
                        for (group_idx, group_data) in tl_data.groups.iter().enumerate() {
                            // Convert geometry points
                            let geometry: Vec<_> = group_data
                                .geom
                                .iter()
                                .map(|p| new_point(p.x, p.y, Some(srid)))
                                .collect();

                            // Convert cell IDs
                            let cells_ids: Vec<i64> = group_data.cells.clone();

                            // Use pre-parsed signals
                            let signals = parsed_signals[tl_idx][group_idx].clone();

                            // Build group
                            let group = TrafficLightGroup::new(group_data.id)
                                .with_label(group_data.label.clone())
                                .with_geometry(geometry)
                                .with_cells_ids(cells_ids)
                                .with_signal(signals)
                                .build();

                            groups.push(group);
                        }

                        // Build traffic light
                        let mut tl_builder = TrafficLight::new(tl_data.id)
                            .with_groups(groups)
                            .with_phases_times(times);

                        // Set coordinates if provided
                        if let Some(geom) = &tl_data.geom {
                            tl_builder = tl_builder.with_coordinates(new_point(geom.x, geom.y, Some(srid)));
                        }

                        let tl = tl_builder.build();
                        session.add_traffic_light(tl);
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
            let resp = pb::SessionTlsResponse {
                code: Code::Ok as u32,
                text: "OK".to_string(),
            };
            if tx.send(Ok(resp)).await.is_err() {
                return;
            }
        }
    });

    let out: BoxStream<pb::SessionTlsResponse> = Box::pin(ReceiverStream::new(rx));
    Ok(Response::new(out))
}
