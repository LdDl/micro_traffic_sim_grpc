"""Decoder for RecordBatch columns blobs (record format version 1).

This mirrors the binary layout produced by the Rust server in
``src/server/record.rs`` (``BatchAcc::to_blob``) and documented in
``protos/record.proto``. End users decode a blob via a single call to
``decode_record_batch`` and receive a per-tick list of the SAME proto
``SessionStepResponse`` messages the live step RPC returns, so replaying a
recording yields the same types as live stepping.

Blob layout (all little-endian), version 1:

    u8   version            (must equal 1)
    u32  tick_start
    u32  tick_count  (K)
    u32  total_rows  (R)
    u32  rows_per_tick[K]
    u32  vehicle_id[R]
    u32  cell[R]
    u8   agent_type[R]
    u16  angle_cdeg[R]       (round(degrees*100), normalized to [0,360))
    i16  speed[R]
    u32  trip_id[R]
    u32  ic_off[R]           (cumulative END offsets)
    u32  ic_vals[ic_off[R-1]]
    u32  tail_off[R]
    u32  tail_vals[tail_off[R-1]]
    u32  tl_group_count (G)
    { u32 tl_id; u32 group_id } * G
    u8   signal[K*G]         (tick-major)
"""

from __future__ import annotations

import struct

from .step_pb2 import (
    AgentType,
    SessionStepResponse,
    TLGroup,
    TLSState,
    VehicleState,
)


# Decoded signal-byte to string char (matches server convention).
_SIGNAL_STR = {
    0: "undefined",
    1: "r",
    2: "y",
    3: "g",
    4: "G",
    5: "s",
    6: "u",
    7: "o",
    8: "O",
}


def decode_record_batch(columns: bytes) -> list[SessionStepResponse]:
    """Decode a RecordBatch columns blob into a per-tick list of responses.

    Returns one ``SessionStepResponse`` per tick in the batch (in order),
    each populated exactly like the live step RPC: ``timestamp`` is the
    absolute tick, ``vehicle_data`` holds ``VehicleState`` messages and
    ``tls_data`` holds ``TLSState`` messages (grouped by traffic-light id).

    Raises ``ValueError`` if the version byte is not 1 or if the blob is
    truncated. Empty batches (no ticks, no vehicles and/or no traffic
    lights) decode without error.
    """
    o = 0

    def rd(fmt: str, size: int):
        nonlocal o
        try:
            v = struct.unpack_from(fmt, columns, o)[0]
        except struct.error as exc:
            raise ValueError("truncated record batch blob") from exc
        o += size
        return v

    try:
        version = struct.unpack_from("<B", columns, o)[0]
    except struct.error as exc:
        raise ValueError("truncated record batch blob") from exc
    o += 1
    if version != 1:
        raise ValueError(f"unsupported record version: {version}")

    tick_start = rd("<I", 4)
    tick_count = rd("<I", 4)
    r = rd("<I", 4)

    rows_per_tick = [rd("<I", 4) for _ in range(tick_count)]
    if sum(rows_per_tick) != r:
        raise ValueError(f"rows_per_tick sum {sum(rows_per_tick)} != total_rows {r}")
    veh_id = [rd("<I", 4) for _ in range(r)]
    cell = [rd("<I", 4) for _ in range(r)]
    if o + r > len(columns):
        raise ValueError("truncated record batch blob")
    agent_type = columns[o:o + r]
    o += r
    angle = [rd("<H", 2) for _ in range(r)]
    speed = [rd("<h", 2) for _ in range(r)]
    trip = [rd("<I", 4) for _ in range(r)]
    ic_off = [rd("<I", 4) for _ in range(r)]
    ic_vals = [rd("<I", 4) for _ in range(ic_off[-1] if r else 0)]
    tail_off = [rd("<I", 4) for _ in range(r)]
    tail_vals = [rd("<I", 4) for _ in range(tail_off[-1] if r else 0)]

    g_count = rd("<I", 4)
    tl_keys = [(rd("<I", 4), rd("<I", 4)) for _ in range(g_count)]
    if o + tick_count * g_count > len(columns):
        raise ValueError("truncated record batch blob")
    tl_signals = columns[o:o + tick_count * g_count]
    o += tick_count * g_count

    def slice_vals(off: list[int], vals: list[int], idx: int) -> list[int]:
        start = off[idx - 1] if idx > 0 else 0
        return list(vals[start:off[idx]])

    responses: list[SessionStepResponse] = []
    row = 0
    for t in range(tick_count):
        resp = SessionStepResponse(
            code=0,
            text="",
            timestamp=tick_start + t,
        )

        for _ in range(rows_per_tick[t]):
            resp.vehicle_data.append(VehicleState(
                vehicle_id=veh_id[row],
                vehicle_type=AgentType.ValueType(agent_type[row]),
                bearing=angle[row] / 100.0,
                speed=speed[row],
                cell=cell[row],
                intermediate_cells=slice_vals(ic_off, ic_vals, row),
                travel_time=-1,
                trip_id=trip[row],
                tail_cells=slice_vals(tail_off, tail_vals, row),
            ))
            row += 1

        # Group this tick's signals by tl_id. Keys are sorted by
        # (tl_id, group_id), so equal-tl_id keys are contiguous.
        current: TLSState | None = None
        for gi in range(g_count):
            tl_id, group_id = tl_keys[gi]
            signal = _SIGNAL_STR.get(tl_signals[t * g_count + gi], "undefined")
            if current is None or current.id != tl_id:
                current = resp.tls_data.add(id=tl_id)
            current.groups.append(TLGroup(id=group_id, signal=signal))

        responses.append(resp)

    return responses
