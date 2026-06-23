//! Decoder for the RecordBatch columns blob.
//!
//! RunAndRecord streams batches of simulation state as an opaque little-endian
//! byte blob in the RecordBatch.columns field. This module turns that blob into
//! the SAME proto/gRPC types that the live `simulation_step_session` RPC
//! returns, so replaying a recording yields identical types to live stepping.
//!
//! The authoritative blob layout (version 1) is produced by the server in
//! src/server/record.rs (BatchAcc::to_blob) and documented in
//! protos/record.proto. All integers are little-endian.
//!
//! USAGE
//!
//! ```rust,no_run
//! use micro_traffic_sim::record::decode_record_batch;
//!
//! fn handle(columns: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//!   let responses = decode_record_batch(columns)?;
//!   for resp in &responses {
//!       for v in &resp.vehicle_data {
//!           println!("tick {} vehicle {} at cell {}", resp.timestamp, v.vehicle_id, v.cell);
//!       }
//!       for tls in &resp.tls_data {
//!           for group in &tls.groups {
//!               println!("tick {} tl {} group {} signal {}", resp.timestamp, tls.id, group.id, group.signal);
//!           }
//!       }
//!   }
//!   Ok(())
//! }
//! ```

use std::convert::TryInto;
use std::error::Error;
use std::fmt;

use crate::pb;

/// Layout version this decoder understands. Blobs carrying any other version
/// are rejected with DecodeError::UnsupportedVersion.
const SUPPORTED_VERSION: u8 = 1;

/// Reasons decoding a RecordBatch columns blob can fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The blob declared a layout version this decoder does not support. The
    /// payload is the version byte that was found.
    UnsupportedVersion(u8),
    /// The blob ended before all declared fields could be read.
    Truncated,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "unsupported record batch version {v} (expected {SUPPORTED_VERSION})"
                )
            }
            DecodeError::Truncated => write!(f, "record batch blob is truncated"),
        }
    }
}

impl Error for DecodeError {}

/// Bounds-checked little-endian cursor over the blob.
struct Reader<'a> {
    buf: &'a [u8],
    off: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, off: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let end = self.off.checked_add(n).ok_or(DecodeError::Truncated)?;
        let slice = self.buf.get(self.off..end).ok_or(DecodeError::Truncated)?;
        self.off = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes(b.try_into().unwrap()))
    }

    fn i16(&mut self) -> Result<i16, DecodeError> {
        let b = self.take(2)?;
        Ok(i16::from_le_bytes(b.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }
}

/// Resolve the END-offset based variable-length slice for one row.
///
/// OFF holds cumulative END offsets, so row i spans vals[off[i-1] .. off[i]]
/// with off[-1] treated as 0. Returned as i64 cells for the proto fields.
fn row_slice(off: &[usize], vals: &[u32], row: usize) -> Result<Vec<i64>, DecodeError> {
    let start = if row == 0 { 0 } else { off[row - 1] };
    let end = off[row];
    let slice = vals.get(start..end).ok_or(DecodeError::Truncated)?;
    Ok(slice.iter().map(|&c| c as i64).collect())
}

/// Map a packed signal code to its one-character display string.
fn signal_string(code: u8) -> String {
    match code {
        1 => "r",
        2 => "y",
        3 => "g",
        4 => "G",
        5 => "s",
        6 => "u",
        7 => "o",
        8 => "O",
        _ => "undefined",
    }
    .to_string()
}

/// Decode a RecordBatch columns blob into per-tick proto SessionStepResponse
/// values, one element per tick in the batch.
///
/// Each response mirrors what the live `simulation_step_session` RPC returns:
/// `timestamp` is the absolute tick, `vehicle_data` holds the tick's
/// VehicleState rows, and `tls_data` groups the tick's signals by traffic
/// light id into TlsState/TlGroup. `code` is 0 and `text` is empty.
///
/// Validates the leading version byte and rejects anything other than version
/// 1 with DecodeError::UnsupportedVersion. A blob that ends early yields
/// DecodeError::Truncated. Empty batches (no ticks, no vehicle rows, and/or no
/// traffic light groups) decode without error.
pub fn decode_record_batch(columns: &[u8]) -> Result<Vec<pb::SessionStepResponse>, DecodeError> {
    let mut rd = Reader::new(columns);

    let version = rd.u8()?;
    if version != SUPPORTED_VERSION {
        return Err(DecodeError::UnsupportedVersion(version));
    }

    let tick_start = rd.u32()?;
    let tick_count = rd.u32()? as usize;
    let total_rows = rd.u32()? as usize;

    let mut rows_per_tick = Vec::with_capacity(tick_count);
    for _ in 0..tick_count {
        rows_per_tick.push(rd.u32()? as usize);
    }

    let mut veh_id = Vec::with_capacity(total_rows);
    for _ in 0..total_rows {
        veh_id.push(rd.u32()?);
    }
    let mut cell = Vec::with_capacity(total_rows);
    for _ in 0..total_rows {
        cell.push(rd.u32()?);
    }
    let agent_type = rd.take(total_rows)?.to_vec();
    let mut angle = Vec::with_capacity(total_rows);
    for _ in 0..total_rows {
        angle.push(rd.u16()?);
    }
    let mut speed = Vec::with_capacity(total_rows);
    for _ in 0..total_rows {
        speed.push(rd.i16()?);
    }
    let mut trip = Vec::with_capacity(total_rows);
    for _ in 0..total_rows {
        trip.push(rd.u32()?);
    }

    let mut ic_off = Vec::with_capacity(total_rows);
    for _ in 0..total_rows {
        ic_off.push(rd.u32()? as usize);
    }
    let ic_total = ic_off.last().copied().unwrap_or(0);
    let mut ic_vals = Vec::with_capacity(ic_total);
    for _ in 0..ic_total {
        ic_vals.push(rd.u32()?);
    }

    let mut tail_off = Vec::with_capacity(total_rows);
    for _ in 0..total_rows {
        tail_off.push(rd.u32()? as usize);
    }
    let tail_total = tail_off.last().copied().unwrap_or(0);
    let mut tail_vals = Vec::with_capacity(tail_total);
    for _ in 0..tail_total {
        tail_vals.push(rd.u32()?);
    }

    let g_count = rd.u32()? as usize;
    let mut tl_keys = Vec::with_capacity(g_count);
    for _ in 0..g_count {
        let tl_id = rd.u32()?;
        let group_id = rd.u32()?;
        tl_keys.push((tl_id, group_id));
    }
    let tl_signals = rd.take(tick_count.saturating_mul(g_count))?.to_vec();

    // One SessionStepResponse per tick, mirroring the live step RPC.
    let mut responses = Vec::with_capacity(tick_count);
    let mut row = 0usize;
    for (t, &n) in rows_per_tick.iter().enumerate() {
        let timestamp = tick_start as i64 + t as i64;

        // Vehicle rows for this tick, in stored order.
        let mut vehicle_data = Vec::with_capacity(n);
        for _ in 0..n {
            if row >= total_rows {
                return Err(DecodeError::Truncated);
            }
            vehicle_data.push(pb::VehicleState {
                vehicle_id: veh_id[row] as i64,
                // Agent codes align exactly with the proto AgentType enum; prost
                // represents proto enums as i32 in the struct field.
                vehicle_type: agent_type[row] as i32,
                bearing: angle[row] as f64 / 100.0,
                speed: speed[row] as i64,
                cell: cell[row] as i64,
                intermediate_cells: row_slice(&ic_off, &ic_vals, row)?,
                // travel_time is intentionally NOT recorded in the blob.
                travel_time: -1,
                trip_id: trip[row] as i64,
                tail_cells: row_slice(&tail_off, &tail_vals, row)?,
            });
            row += 1;
        }

        // Signals for this tick, grouped by tl_id. Keys are sorted by
        // (tl_id, group_id), so same-tl_id keys are contiguous: walk them in
        // order and start a new TlsState whenever tl_id changes.
        let mut tls_data: Vec<pb::TlsState> = Vec::new();
        for (gi, &(tl_id, group_id)) in tl_keys.iter().enumerate() {
            let code = tl_signals[t * g_count + gi];
            let group = pb::TlGroup {
                id: group_id as i64,
                signal: signal_string(code),
            };
            match tls_data.last_mut() {
                Some(last) if last.id == tl_id as i64 => last.groups.push(group),
                _ => tls_data.push(pb::TlsState {
                    id: tl_id as i64,
                    groups: vec![group],
                }),
            }
        }

        responses.push(pb::SessionStepResponse {
            code: 0,
            text: String::new(),
            timestamp,
            vehicle_data,
            tls_data,
        });
    }

    Ok(responses)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hand-build a tiny version-1 blob with 2 ticks, 3 vehicle rows total, and
    // 2 traffic light groups, then round-trip it through the decoder.
    fn build_blob() -> Vec<u8> {
        let mut b = Vec::new();
        // version
        b.push(1u8);
        // tick_start
        b.extend_from_slice(&100u32.to_le_bytes());
        // tick_count K = 2
        b.extend_from_slice(&2u32.to_le_bytes());
        // total_rows R = 3
        b.extend_from_slice(&3u32.to_le_bytes());
        // rows_per_tick[K]: 2 rows at tick 100, 1 row at tick 101
        b.extend_from_slice(&2u32.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes());
        // vehicle_id[R]
        for v in [10u32, 11, 12] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // cell[R]
        for v in [5u32, 6, 7] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // agent_type[R]
        b.extend_from_slice(&[1u8, 2, 5]);
        // angle_cdeg[R] -> 0.00, 90.00, 359.99
        for v in [0u16, 9000, 35999] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // speed[R]
        for v in [3i16, 0, -1] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // trip_id[R]
        for v in [1u32, 1, 2] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // ic_off[R] cumulative END offsets: row0 [100,101], row1 [], row2 [102]
        for v in [2u32, 2, 3] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // ic_vals
        for v in [100u32, 101, 102] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // tail_off[R]: row0 [], row1 [200], row2 [201,202]
        for v in [0u32, 1, 3] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // tail_vals
        for v in [200u32, 201, 202] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // tl_group_count G = 2
        b.extend_from_slice(&2u32.to_le_bytes());
        // keys (tl_id, group_id) sorted ascending; both share tl_id 1
        b.extend_from_slice(&1u32.to_le_bytes());
        b.extend_from_slice(&100u32.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes());
        b.extend_from_slice(&200u32.to_le_bytes());
        // signal[K*G] tick-major: t0 -> [g(3), r(1)], t1 -> [r(1), g(3)]
        b.extend_from_slice(&[3u8, 1, 1, 3]);
        b
    }

    #[test]
    fn round_trip_blob() {
        let blob = build_blob();
        let responses = decode_record_batch(&blob).expect("decode");

        assert_eq!(responses.len(), 2);

        // Tick 100: two vehicles, both TL groups for tl 1.
        let r0 = &responses[0];
        assert_eq!(r0.code, 0);
        assert_eq!(r0.text, "");
        assert_eq!(r0.timestamp, 100);
        assert_eq!(r0.vehicle_data.len(), 2);

        let v0 = &r0.vehicle_data[0];
        assert_eq!(v0.vehicle_id, 10);
        assert_eq!(v0.cell, 5);
        assert_eq!(v0.vehicle_type, pb::AgentType::Car as i32);
        assert!((v0.bearing - 0.0).abs() < 1e-9);
        assert_eq!(v0.speed, 3);
        assert_eq!(v0.trip_id, 1);
        assert_eq!(v0.travel_time, -1);
        assert_eq!(v0.intermediate_cells, vec![100i64, 101]);
        assert_eq!(v0.tail_cells, Vec::<i64>::new());

        let v1 = &r0.vehicle_data[1];
        assert_eq!(v1.vehicle_id, 11);
        assert_eq!(v1.vehicle_type, pb::AgentType::Bus as i32);
        assert!((v1.bearing - 90.0).abs() < 1e-9);
        assert_eq!(v1.speed, 0);
        assert_eq!(v1.intermediate_cells, Vec::<i64>::new());
        assert_eq!(v1.tail_cells, vec![200i64]);

        // Both groups collapse into one TLSState for tl_id 1.
        assert_eq!(r0.tls_data.len(), 1);
        assert_eq!(r0.tls_data[0].id, 1);
        assert_eq!(r0.tls_data[0].groups.len(), 2);
        assert_eq!(r0.tls_data[0].groups[0].id, 100);
        assert_eq!(r0.tls_data[0].groups[0].signal, "g");
        assert_eq!(r0.tls_data[0].groups[1].id, 200);
        assert_eq!(r0.tls_data[0].groups[1].signal, "r");

        // Tick 101: one vehicle, signals flipped.
        let r1 = &responses[1];
        assert_eq!(r1.timestamp, 101);
        assert_eq!(r1.vehicle_data.len(), 1);

        let v2 = &r1.vehicle_data[0];
        assert_eq!(v2.vehicle_id, 12);
        assert_eq!(v2.vehicle_type, pb::AgentType::Truck as i32);
        assert!((v2.bearing - 359.99).abs() < 1e-9);
        assert_eq!(v2.speed, -1);
        assert_eq!(v2.trip_id, 2);
        assert_eq!(v2.intermediate_cells, vec![102i64]);
        assert_eq!(v2.tail_cells, vec![201i64, 202]);

        assert_eq!(r1.tls_data.len(), 1);
        assert_eq!(r1.tls_data[0].id, 1);
        assert_eq!(r1.tls_data[0].groups[0].signal, "r");
        assert_eq!(r1.tls_data[0].groups[1].signal, "g");
    }

    #[test]
    fn rejects_bad_version() {
        let mut blob = build_blob();
        blob[0] = 2;
        match decode_record_batch(&blob) {
            Err(DecodeError::UnsupportedVersion(2)) => {}
            other => panic!("expected UnsupportedVersion(2), got {other:?}"),
        }
    }

    #[test]
    fn rejects_truncated() {
        let blob = build_blob();
        match decode_record_batch(&blob[..5]) {
            Err(DecodeError::Truncated) => {}
            other => panic!("expected Truncated, got {other:?}"),
        }
    }

    #[test]
    fn empty_batch_no_panic() {
        // version, tick_start, tick_count=0, total_rows=0, g_count=0
        let mut b = Vec::new();
        b.push(1u8);
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        let responses = decode_record_batch(&b).expect("decode empty");
        assert!(responses.is_empty());
    }
}
