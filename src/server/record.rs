use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use micro_traffic_sim::pb;
use micro_traffic_sim_core::agents_types::AgentType;
use micro_traffic_sim_core::simulation::sessions_storage::SessionsStorage;
use micro_traffic_sim_core::simulation::states::VehicleState;

use super::BoxStream;

/// Layout version of the RecordBatch.columns blob. See `protos/record.proto`
/// RECORD BLOB LAYOUT. Bump on ANY change to the blob layout.
const RECORD_BATCH_VERSION: u8 = 1;
/// Ticks per RecordBatch when the request leaves `batch_ticks = 0`.
const DEFAULT_BATCH_TICKS: u32 = 300;
/// Safety cap on the number of ticks when `horizon_ticks = 0` (run until drained).
const HORIZON_HARD_CAP: u64 = 1_000_000;
/// Mirrors the (private) core SPAWN_SEED used to seed vehicle spawning, recorded
/// into RunMetadata for reproducibility.
const SPAWN_SEED: u64 = 0x00C0_FFEE;
/// Simulated seconds per tick (the core advances at 1 s/tick).
const TICK_SECONDS: f64 = 1.0;

/// Column-major accumulator for one RecordBatch. Mirrors the RECORD BLOB LAYOUT;
/// `to_blob` emits the opaque little-endian blob carried in RecordBatch.columns.
#[derive(Default)]
struct BatchAcc {
    tick_start: u32,
    rows_per_tick: Vec<u32>,
    veh_id: Vec<u32>,
    cell: Vec<u32>,
    vtype: Vec<u8>,
    angle: Vec<u16>,
    speed: Vec<i16>,
    trip: Vec<u32>,
    // cumulative end offset into ic_vals, one per row
    ic_off: Vec<u32>,
    ic_vals: Vec<u32>,
    // cumulative end offset into tail_vals, one per row
    tail_off: Vec<u32>,
    tail_vals: Vec<u32>,
}

impl BatchAcc {
    fn is_empty(&self) -> bool {
        self.rows_per_tick.is_empty()
    }

    fn ticks(&self) -> u32 {
        self.rows_per_tick.len() as u32
    }

    fn rows(&self) -> u32 {
        self.veh_id.len() as u32
    }

    fn clear(&mut self) {
        self.tick_start = 0;
        self.rows_per_tick.clear();
        self.veh_id.clear();
        self.cell.clear();
        self.vtype.clear();
        self.angle.clear();
        self.speed.clear();
        self.trip.clear();
        self.ic_off.clear();
        self.ic_vals.clear();
        self.tail_off.clear();
        self.tail_vals.clear();
    }

    fn push_tick(&mut self, vehicles: &[VehicleState]) {
        self.rows_per_tick.push(vehicles.len() as u32);
        for v in vehicles {
            self.veh_id.push(v.id as u32);
            self.cell.push(v.last_cell as u32);
            self.vtype.push(agent_type_u8(v.vehicle_type));
            self.angle
                .push((v.last_angle.rem_euclid(360.0) * 100.0) as u16);
            self.speed.push(v.last_speed as i16);
            self.trip.push(v.trip_id as u32);
            for &c in &v.last_intermediate_cells {
                self.ic_vals.push(c as u32);
            }
            self.ic_off.push(self.ic_vals.len() as u32);
            for &t in &v.tail_cells {
                self.tail_vals.push(t as u32);
            }
            self.tail_off.push(self.tail_vals.len() as u32);
        }
    }

    fn to_blob(&self) -> Vec<u8> {
        let total_rows = self.veh_id.len();
        let mut buf = Vec::with_capacity(
            13 + self.rows_per_tick.len() * 4
                + total_rows * 25
                + (self.ic_vals.len() + self.tail_vals.len()) * 4,
        );
        buf.push(RECORD_BATCH_VERSION);
        buf.extend_from_slice(&self.tick_start.to_le_bytes());
        buf.extend_from_slice(&(self.rows_per_tick.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(total_rows as u32).to_le_bytes());
        for &r in &self.rows_per_tick {
            buf.extend_from_slice(&r.to_le_bytes());
        }
        for &x in &self.veh_id {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        for &x in &self.cell {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        buf.extend_from_slice(&self.vtype);
        for &x in &self.angle {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        for &x in &self.speed {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        for &x in &self.trip {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        for &x in &self.ic_off {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        for &x in &self.ic_vals {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        for &x in &self.tail_off {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        for &x in &self.tail_vals {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        buf
    }

    fn into_proto(&self) -> pb::RecordBatch {
        pb::RecordBatch {
            tick_start: self.tick_start as u64,
            tick_count: self.ticks(),
            total_rows: self.rows(),
            columns: self.to_blob(),
        }
    }
}

fn agent_type_u8(a: AgentType) -> u8 {
    match a {
        AgentType::Undefined => 0,
        AgentType::Car => 1,
        AgentType::Bus => 2,
        AgentType::Taxi => 3,
        AgentType::Pedestrian => 4,
        AgentType::Truck => 5,
        AgentType::LargeBus => 6,
    }
}

/// Self-describing column layout matching the RECORD BLOB LAYOUT (in order).
fn column_schema() -> pb::ColumnSchema {
    let col = |name: &str, ty: &str| pb::ColumnDef {
        name: name.to_string(),
        r#type: ty.to_string(),
    };
    pb::ColumnSchema {
        columns: vec![
            col("vehicle_id", "u32"),
            col("cell", "u32"),
            col("agent_type", "u8"),
            col("angle_cdeg", "u16"),
            col("speed", "i16"),
            col("trip_id", "u32"),
            col("intermediate_cells", "list<u32>"),
            col("tail_cells", "list<u32>"),
        ],
    }
}

fn wrap(payload: pb::run_and_record_response::Payload) -> pb::RunAndRecordResponse {
    pb::RunAndRecordResponse {
        payload: Some(payload),
    }
}

/// Headless run + columnar trajectory recording. See `protos/record.proto` for the
/// wire contract and the RECORD BLOB LAYOUT.
///
/// Advances the session forward with no per-tick client round-trip and streams the
/// per-tick `VehicleState` dump back as versioned column-major batches: RunMetadata
/// first, then one RecordBatch every `batch_ticks` (K) ticks, then RunSummary last.
/// The sessions lock is re-acquired per tick (never held across the whole run nor
/// across an await), and responses go through a bounded mpsc channel for natural
/// backpressure so a slow consumer cannot make the producer outrun memory.
pub async fn run_and_record(
    sessions: Arc<Mutex<SessionsStorage>>,
    request: Request<pb::RunAndRecordRequest>,
) -> Result<Response<BoxStream<pb::RunAndRecordResponse>>, Status> {
    let req = request.into_inner();

    let session_id = req
        .session_id
        .as_ref()
        .map(|u| u.value.clone())
        .ok_or_else(|| Status::invalid_argument("No session ID has been provided"))?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|_| {
        Status::invalid_argument(format!("Session ID should be a UUID v4: '{}'", session_id))
    })?;

    let batch_ticks = (if req.batch_ticks == 0 {
        DEFAULT_BATCH_TICKS
    } else {
        req.batch_ticks
    }) as usize;
    let explicit_horizon = req.horizon_ticks > 0;
    let max_ticks = if explicit_horizon {
        req.horizon_ticks
    } else {
        HORIZON_HARD_CAP
    };
    // Best-effort: the per-tick stochastic seed (MTSC_SEED) is entropy-seeded in the
    // core when unset, in which case the exact seed used is not recoverable here.
    let stochastic_seed = std::env::var("MTSC_SEED")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let (tx, rx) = mpsc::channel::<Result<pb::RunAndRecordResponse, Status>>(16);

    tokio::spawn(async move {
        // Metadata, sent exactly once before any batch.
        let meta = pb::RunMetadata {
            format_version: RECORD_BATCH_VERSION as u32,
            tick_seconds: TICK_SECONDS,
            spawn_seed: SPAWN_SEED,
            stochastic_seed,
            // @todo: surface micro_traffic_sim_core version / commit
            core_version: String::new(),
            // @todo: surface the rand crate version
            rand_version: String::new(),
            // @todo: hash grid + trips + TLS + routing options
            config_hash: String::new(),
            schema: Some(column_schema()),
        };
        if tx
            .send(Ok(wrap(pb::run_and_record_response::Payload::Metadata(
                meta,
            ))))
            .await
            .is_err()
        {
            return;
        }

        // Step the engine, packing batches.
        let mut batch = BatchAcc::default();
        let mut total_ticks: u64 = 0;
        let mut total_rows: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut completed: i32 = 0;
        let mut lost: i32 = 0;
        let mut seen_any = false;

        for _ in 0..max_ticks {
            // Step under the lock; the guard is fully dropped before any await
            // (the match yields a Send value, so no MutexGuard crosses an await).
            let stepped = match sessions.lock() {
                Ok(mut guard) => Ok(guard.with_session_mut(&session_uuid, |s| s.step())),
                Err(_) => Err(()),
            };

            let dump = match stepped {
                Err(()) => {
                    let _ = tx
                        .send(Err(Status::internal("sessions storage lock poisoned")))
                        .await;
                    return;
                }
                Ok(None) => {
                    let _ = tx
                        .send(Err(Status::not_found(format!(
                            "Not found session ID: '{}'",
                            session_id
                        ))))
                        .await;
                    return;
                }
                Ok(Some(Err(e))) => {
                    let _ = tx.send(Err(Status::aborted(e.to_string()))).await;
                    return;
                }
                Ok(Some(Ok(d))) => d,
            };

            if batch.is_empty() {
                batch.tick_start = dump.timestamp as u32;
            }
            batch.push_tick(&dump.vehicles);

            let n = dump.vehicles.len();
            if n > 0 {
                seen_any = true;
            }
            total_ticks += 1;
            total_rows += n as u64;
            completed = dump.vehicles_completed;
            lost = dump.vehicles_lost;

            // Flush a full batch.
            if batch.ticks() as usize >= batch_ticks {
                let rb = batch.into_proto();
                total_bytes += rb.columns.len() as u64;
                if tx
                    .send(Ok(wrap(pb::run_and_record_response::Payload::Batch(rb))))
                    .await
                    .is_err()
                {
                    return;
                }
                batch.clear();
            }

            // Open-horizon mode stops once the network has drained.
            if !explicit_horizon && seen_any && n == 0 {
                break;
            }
        }

        // Flush the trailing partial batch.
        if !batch.is_empty() {
            let rb = batch.into_proto();
            total_bytes += rb.columns.len() as u64;
            if tx
                .send(Ok(wrap(pb::run_and_record_response::Payload::Batch(rb))))
                .await
                .is_err()
            {
                return;
            }
        }

        // Summary, sent exactly once after the last batch.
        let summary = pb::RunSummary {
            total_ticks,
            total_rows,
            total_bytes,
            vehicles_completed: completed,
            vehicles_lost: lost,
        };
        let _ = tx
            .send(Ok(wrap(pb::run_and_record_response::Payload::Summary(
                summary,
            ))))
            .await;
    });

    let out: BoxStream<pb::RunAndRecordResponse> = Box::pin(ReceiverStream::new(rx));
    Ok(Response::new(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn veh(
        id: u64,
        cell: i64,
        speed: i32,
        angle: f64,
        trip: i64,
        ic: Vec<i64>,
        tail: Vec<i64>,
        t: AgentType,
    ) -> VehicleState {
        VehicleState {
            occupied_points: vec![],
            last_point: [0.0, 0.0],
            last_cell: cell,
            tail_cells: tail,
            last_intermediate_cells: ic,
            last_speed: speed,
            last_angle: angle,
            vehicle_type: t,
            travel_time: 0,
            id,
            trip_id: trip,
        }
    }

    /// Round-trips the RECORD BLOB LAYOUT: pack two ticks, then decode the blob
    /// field by field and assert it matches (incl. the var-length ic/tail offsets).
    #[test]
    fn blob_roundtrip() {
        let rd_u32 = |b: &[u8], o: &mut usize| -> u32 {
            let v = u32::from_le_bytes(b[*o..*o + 4].try_into().unwrap());
            *o += 4;
            v
        };
        let rd_u16 = |b: &[u8], o: &mut usize| -> u16 {
            let v = u16::from_le_bytes(b[*o..*o + 2].try_into().unwrap());
            *o += 2;
            v
        };
        let rd_i16 = |b: &[u8], o: &mut usize| -> i16 {
            let v = i16::from_le_bytes(b[*o..*o + 2].try_into().unwrap());
            *o += 2;
            v
        };

        let mut acc = BatchAcc::default();
        acc.tick_start = 7;
        // tick 1: a car (no ic, no tail) + a bus (ic=[10,11], tail=[20])
        acc.push_tick(&[
            veh(1, 100, 0, 0.0, 5, vec![], vec![], AgentType::Car),
            veh(2, 101, 3, 90.0, 5, vec![10, 11], vec![20], AgentType::Bus),
        ]);
        // tick 2: just the car
        acc.push_tick(&[veh(1, 102, 1, 180.0, 5, vec![], vec![], AgentType::Car)]);

        assert_eq!(acc.ticks(), 2);
        assert_eq!(acc.rows(), 3);

        let blob = acc.to_blob();
        let mut o = 0usize;

        // header
        assert_eq!(blob[o], RECORD_BATCH_VERSION);
        o += 1;
        assert_eq!(rd_u32(&blob, &mut o), 7); // tick_start
        assert_eq!(rd_u32(&blob, &mut o), 2); // tick_count (K)
        assert_eq!(rd_u32(&blob, &mut o), 3); // total_rows (R)

        // rows_per_tick[2]
        assert_eq!(rd_u32(&blob, &mut o), 2);
        assert_eq!(rd_u32(&blob, &mut o), 1);

        // vehicle_id[3]
        assert_eq!(
            [
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o)
            ],
            [1, 2, 1]
        );
        // cell[3]
        assert_eq!(
            [
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o)
            ],
            [100, 101, 102]
        );
        // agent_type[3] (u8): Car=1, Bus=2, Car=1
        assert_eq!(&blob[o..o + 3], &[1u8, 2, 1]);
        o += 3;
        // angle_cdeg[3] (u16)
        assert_eq!(
            [
                rd_u16(&blob, &mut o),
                rd_u16(&blob, &mut o),
                rd_u16(&blob, &mut o)
            ],
            [0, 9000, 18000]
        );
        // speed[3] (i16)
        assert_eq!(
            [
                rd_i16(&blob, &mut o),
                rd_i16(&blob, &mut o),
                rd_i16(&blob, &mut o)
            ],
            [0, 3, 1]
        );
        // trip_id[3]
        assert_eq!(
            [
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o)
            ],
            [5, 5, 5]
        );

        // ic_off[3]: car 0, bus +2 = 2, car +0 = 2
        assert_eq!(
            [
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o)
            ],
            [0, 2, 2]
        );
        // ic_vals[2]: 10, 11
        assert_eq!([rd_u32(&blob, &mut o), rd_u32(&blob, &mut o)], [10, 11]);
        // tail_off[3]: 0, 1, 1
        assert_eq!(
            [
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o),
                rd_u32(&blob, &mut o)
            ],
            [0, 1, 1]
        );
        // tail_vals[1]: 20
        assert_eq!(rd_u32(&blob, &mut o), 20);

        // blob is exactly consumed
        assert_eq!(o, blob.len());
    }
}
