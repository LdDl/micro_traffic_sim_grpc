from __future__ import annotations

import math
import os
import struct
import sys
from dataclasses import dataclass
from typing import Iterator

import grpc

from micro_traffic_sim import (
    AgentType,
    BehaviourType,
    Cell,
    ConflictWinnerType,
    ConflictZone,
    ConflictZoneType,
    Group,
    GroupType,
    Point,
    RunAndRecordRequest,
    ServiceStub,
    SessionConflictZones,
    SessionGrid,
    SessionReq,
    SessionTLS,
    SessionTrip,
    TrafficLight,
    Trip,
    TripType,
    UUIDv4,
    ZoneType,
)


@dataclass
class CellData:
    id: int
    x: float
    y: float
    forward_node: int
    left_node: int
    right_node: int
    zone_type: ZoneType


def zone_type_str(z: ZoneType) -> str:
    mapping = {
        ZoneType.ZONE_TYPE_BIRTH: "birth",
        ZoneType.ZONE_TYPE_DEATH: "death",
        ZoneType.ZONE_TYPE_COORDINATION: "coordination",
        ZoneType.ZONE_TYPE_COMMON: "common",
        ZoneType.ZONE_TYPE_ISOLATED: "isolated",
        ZoneType.ZONE_TYPE_LANE_FOR_BUS: "lane_for_bus",
        ZoneType.ZONE_TYPE_TRANSIT: "transit",
        ZoneType.ZONE_TYPE_CROSSWALK: "crosswalk",
    }
    return mapping.get(z, "undefined")


def main() -> None:
    raw_addr = os.environ.get("MT_SIM_ADDR", "127.0.0.1:50051")
    addr = raw_addr.removeprefix("http://").removeprefix("https://")
    channel = grpc.insecure_channel(addr)
    client = ServiceStub(channel)

    new_resp = client.NewSession(SessionReq(srid=0))
    if new_resp.id is None:
        raise RuntimeError("Server returned empty session id")
    session_id = new_resp.id.value
    print(f"session: {session_id}", file=sys.stderr)

    cells: list[Cell] = []
    cell_data: list[CellData] = []
    cell_coords: dict[int, tuple[float, float]] = {}

    for i in range(10):
        zone_type = ZoneType.ZONE_TYPE_COMMON
        if i == 0:
            zone_type = ZoneType.ZONE_TYPE_BIRTH
        elif i == 9:
            zone_type = ZoneType.ZONE_TYPE_DEATH

        forward_node = i + 1 if i < 9 else -1
        left_node = -1
        if i == 3:
            left_node = 14
        elif i == 6:
            left_node = 24

        x, y = float(i), 3.5
        cells.append(Cell(
            id=i,
            geom=Point(x=x, y=y),
            zone_type=zone_type,
            speed_limit=1,
            left_node=left_node,
            forward_node=forward_node,
            right_node=-1,
            meso_link_id=0,
        ))
        cell_data.append(CellData(
            id=i, x=x, y=y,
            forward_node=forward_node, left_node=left_node, right_node=-1,
            zone_type=zone_type,
        ))
        cell_coords[i] = (x, y)

    for i in range(10):
        cell_id = 10 + i
        zone_type = ZoneType.ZONE_TYPE_COMMON
        if i == 0:
            zone_type = ZoneType.ZONE_TYPE_BIRTH
        elif i == 9:
            zone_type = ZoneType.ZONE_TYPE_DEATH

        forward_node = cell_id + 1 if i < 9 else -1
        right_node = 4 if i == 3 else -1

        x, y = 3.5, float(i)
        cells.append(Cell(
            id=cell_id,
            geom=Point(x=x, y=y),
            zone_type=zone_type,
            speed_limit=1,
            left_node=-1,
            forward_node=forward_node,
            right_node=right_node,
            meso_link_id=0,
        ))
        cell_data.append(CellData(
            id=cell_id, x=x, y=y,
            forward_node=forward_node, left_node=-1, right_node=right_node,
            zone_type=zone_type,
        ))
        cell_coords[cell_id] = (x, y)

    for i in range(10):
        cell_id = 20 + i
        zone_type = ZoneType.ZONE_TYPE_COMMON
        if i == 0:
            zone_type = ZoneType.ZONE_TYPE_BIRTH
        elif i == 9:
            zone_type = ZoneType.ZONE_TYPE_DEATH

        forward_node = cell_id + 1 if i < 9 else -1
        right_node = 7 if i == 3 else -1

        x, y = 6.5, float(i)
        cells.append(Cell(
            id=cell_id,
            geom=Point(x=x, y=y),
            zone_type=zone_type,
            speed_limit=1,
            left_node=-1,
            forward_node=forward_node,
            right_node=right_node,
            meso_link_id=0,
        ))
        cell_data.append(CellData(
            id=cell_id, x=x, y=y,
            forward_node=forward_node, left_node=-1, right_node=right_node,
            zone_type=zone_type,
        ))
        cell_coords[cell_id] = (x, y)

    def grid_requests() -> Iterator[SessionGrid]:
        yield SessionGrid(
            session_id=UUIDv4(value=session_id),
            data=cells,
        )

    for resp in client.PushSessionGrid(grid_requests()):
        print(f"grid push: code={resp.code} text={resp.text}", file=sys.stderr)

    conflict_zones = [
        ConflictZone(
            id=1,
            source_x=3,
            target_x=4,
            source_y=13,
            target_y=14,
            conflict_winner=ConflictWinnerType.CONFLICT_WINNER_SECOND,
            conflict_type=ConflictZoneType.CONFLICT_ZONE_TYPE_UNDEFINED,
        ),
    ]

    def cz_requests() -> Iterator[SessionConflictZones]:
        yield SessionConflictZones(
            session_id=UUIDv4(value=session_id),
            data=conflict_zones,
        )

    for resp in client.PushSessionConflictZones(cz_requests()):
        print(f"conflict zones push: code={resp.code} text={resp.text}", file=sys.stderr)

    traffic_lights = [
        TrafficLight(
            id=1,
            geom=Point(x=7.0, y=4.0),
            groups=[
                Group(
                    id=100,
                    label="Group block H",
                    cells=[6],
                    signals=["g", "r"],
                    type=GroupType.GROUP_TYPE_VEHICLE,
                    crosswalk_length=0.0,
                ),
                Group(
                    id=200,
                    label="Group block V2",
                    cells=[23],
                    signals=["r", "g"],
                    type=GroupType.GROUP_TYPE_VEHICLE,
                    crosswalk_length=0.0,
                ),
            ],
            times=[5, 5],
        ),
    ]

    tls_group_cells: dict[tuple[int, int], list[int]] = {}
    for tl in traffic_lights:
        for group in tl.groups:
            tls_group_cells[(tl.id, group.id)] = list(group.cells)

    def tls_requests() -> Iterator[SessionTLS]:
        yield SessionTLS(
            session_id=UUIDv4(value=session_id),
            data=traffic_lights,
        )

    for resp in client.PushSessionTLS(tls_requests()):
        print(f"tls push: code={resp.code} text={resp.text}", file=sys.stderr)

    trips = [
        Trip(
            id=1,
            trip_type=TripType.TRIP_TYPE_RANDOM,
            from_node=1,
            to_node=9,
            initial_speed=1,
            probability=0.2,
            agent_type=AgentType.AGENT_TYPE_CAR,
            behaviour_type=BehaviourType.BEHAVIOUR_TYPE_COOPERATIVE,
        ),
        Trip(
            id=2,
            trip_type=TripType.TRIP_TYPE_RANDOM,
            from_node=10,
            to_node=19,
            initial_speed=1,
            probability=0.3,
            agent_type=AgentType.AGENT_TYPE_CAR,
            behaviour_type=BehaviourType.BEHAVIOUR_TYPE_COOPERATIVE,
        ),
        Trip(
            id=3,
            trip_type=TripType.TRIP_TYPE_RANDOM,
            from_node=20,
            to_node=29,
            initial_speed=1,
            probability=0.1,
            agent_type=AgentType.AGENT_TYPE_CAR,
            behaviour_type=BehaviourType.BEHAVIOUR_TYPE_COOPERATIVE,
        ),
    ]

    def trip_requests() -> Iterator[SessionTrip]:
        yield SessionTrip(
            session_id=UUIDv4(value=session_id),
            data=trips,
        )

    for resp in client.PushSessionTrip(trip_requests()):
        print(f"trip push: code={resp.code} text={resp.text}", file=sys.stderr)

    print("tl_id;x;y")
    for tl in traffic_lights:
        if tl.geom is not None:
            print(f"{tl.id};{tl.geom.x:.5f};{tl.geom.y:.5f}")

    print("tl_id;controlled_cell;x;y")
    for tl in traffic_lights:
        for group in tl.groups:
            for cell_id in group.cells:
                if cell_id in cell_coords:
                    x, y = cell_coords[cell_id]
                    print(f"{tl.id};{cell_id};{x:.5f};{y:.5f}")

    print("cell_id;x;y;forward_x;forward_y;connection_type;zone")
    for cd in cell_data:
        print(f"{cd.id};{cd.x:.5f};{cd.y:.5f};{cd.x:.5f};{cd.y:.5f};cell;{zone_type_str(cd.zone_type)}")
    for cd in cell_data:
        if cd.forward_node != -1 and cd.forward_node in cell_coords:
            fx, fy = cell_coords[cd.forward_node]
            print(f"{cd.id};{cd.x:.5f};{cd.y:.5f};{fx:.5f};{fy:.5f};forward;common")
        if cd.left_node != -1 and cd.left_node in cell_coords:
            lx, ly = cell_coords[cd.left_node]
            print(f"{cd.id};{cd.x:.5f};{cd.y:.5f};{lx:.5f};{ly:.5f};left;common")
        if cd.right_node != -1 and cd.right_node in cell_coords:
            rx, ry = cell_coords[cd.right_node]
            print(f"{cd.id};{cd.x:.5f};{cd.y:.5f};{rx:.5f};{ry:.5f};right;common")

    print("\n=== Running 50 simulation steps ===\n")

    tls_rows: list[str] = []
    print("step;vehicle_id;vehicle_type;speed;bearing;intermediate_cells;cell;x;y;tail_cells;trip_id")
    request = RunAndRecordRequest(
        session_id=UUIDv4(value=session_id),
        horizon_ticks=50,
        batch_ticks=20,
    )
    for resp in client.RunAndRecord(request):
        which = resp.WhichOneof("payload")
        if which == "metadata":
            m = resp.metadata
            cols = ",".join(c.name for c in m.schema.columns) if m.HasField("schema") else ""
            print(
                f"format_version={m.format_version} tick_seconds={m.tick_seconds} "
                f"spawn_seed={m.spawn_seed} stochastic_seed={m.stochastic_seed} columns=[{cols}]",
                file=sys.stderr,
            )
        elif which == "batch":
            tls_rows.extend(decode_batch(resp.batch.columns, cell_coords, tls_group_cells))
        elif which == "summary":
            s = resp.summary
            print(
                f"ticks={s.total_ticks} rows={s.total_rows} raw_bytes={s.total_bytes} "
                f"completed={s.vehicles_completed} lost={s.vehicles_lost}",
                file=sys.stderr,
            )

    print("tl_step;tl_id;group_id;cell_id;x;y;signal")
    for row in tls_rows:
        print(row)

    print("\nSimulation complete!")


def decode_batch(
    blob: bytes,
    cell_coords: dict[int, tuple[float, float]],
    tls_group_cells: dict[tuple[int, int], list[int]],
) -> list[str]:
    o = 0

    def rd(fmt: str, size: int):
        nonlocal o
        v = struct.unpack_from(fmt, blob, o)[0]
        o += size
        return v

    o += 1
    tick_start = rd("<I", 4)
    tick_count = rd("<I", 4)
    r = rd("<I", 4)

    rows_per_tick = [rd("<I", 4) for _ in range(tick_count)]
    veh_id = [rd("<I", 4) for _ in range(r)]
    cell = [rd("<I", 4) for _ in range(r)]
    agent_type = blob[o:o + r]
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
    tl_signals = blob[o:o + tick_count * g_count]
    o += tick_count * g_count

    vtypes = {1: "car", 2: "bus", 3: "taxi", 4: "pedestrian", 5: "truck", 6: "large_bus"}

    def sl(off: list[int], vals: list[int], row: int) -> str:
        start = off[row - 1] if row > 0 else 0
        return ",".join(str(v) for v in vals[start:off[row]])

    row = 0
    for t in range(tick_count):
        step = tick_start + t
        for _ in range(rows_per_tick[t]):
            c = cell[row]
            x, y = cell_coords.get(c, (math.nan, math.nan))
            print(
                f"{step};{veh_id[row]};{vtypes.get(agent_type[row], 'undefined')};{speed[row]};"
                f"{angle[row] / 100.0:.2f};{sl(ic_off, ic_vals, row)};{c};{x:.2f};{y:.2f};"
                f"{sl(tail_off, tail_vals, row)};{trip[row]}"
            )
            row += 1

    sigstr = {1: "r", 2: "y", 3: "g", 4: "G", 5: "s", 6: "u", 7: "o", 8: "O"}
    tls_rows: list[str] = []
    for t in range(tick_count):
        step = tick_start + t
        for gi in range(g_count):
            tl_id, group_id = tl_keys[gi]
            sig = sigstr.get(tl_signals[t * g_count + gi], "undefined")
            for cell_id in tls_group_cells.get((tl_id, group_id), []):
                if cell_id in cell_coords:
                    x, y = cell_coords[cell_id]
                    tls_rows.append(f"{step};{tl_id};{group_id};{cell_id};{x:.5f};{y:.5f};{sig}")
    return tls_rows


if __name__ == "__main__":
    main()
