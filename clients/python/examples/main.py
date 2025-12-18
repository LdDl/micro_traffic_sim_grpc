#!/usr/bin/env python3
"""
Complete traffic simulation example using the Python gRPC client.
Matches functionality of Go and Rust examples.
"""

from __future__ import annotations

import math
import os
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
    ServiceStub,
    SessionConflictZones,
    SessionGrid,
    SessionReq,
    SessionStep,
    SessionTLS,
    SessionTrip,
    TLGroup,
    TLSState,
    TrafficLight,
    Trip,
    TripType,
    UUIDv4,
    VehicleState,
    ZoneType,
)


@dataclass
class CellData:
    """Stores cell information for gnuplot output."""
    id: int
    x: float
    y: float
    forward_node: int
    left_node: int
    right_node: int
    zone_type: ZoneType


@dataclass
class VehicleStateRecord:
    """Stores vehicle state for output."""
    step: int
    vehicle_id: int
    vehicle_type: str
    speed: int
    bearing: float
    intermediate_cells: str
    cell: int
    x: float
    y: float
    tail_cells: str
    trip_id: int


@dataclass
class TLSStateRecord:
    """Stores TLS state for output."""
    step: int
    tl_id: int
    group_id: int
    cell_id: int
    x: float
    y: float
    signal: str


def zone_type_str(z: ZoneType) -> str:
    """Convert ZoneType enum to string."""
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


def agent_type_str(a: AgentType) -> str:
    """Convert AgentType enum to string."""
    mapping = {
        AgentType.AGENT_TYPE_CAR: "car",
        AgentType.AGENT_TYPE_BUS: "bus",
        AgentType.AGENT_TYPE_TAXI: "taxi",
        AgentType.AGENT_TYPE_PEDESTRIAN: "pedestrian",
    }
    return mapping.get(a, "undefined")


def main() -> None:
    # Get server address from environment
    raw_addr = os.environ.get("MT_SIM_ADDR", "127.0.0.1:50051")
    addr = raw_addr.removeprefix("http://").removeprefix("https://")

    # Connect to server
    channel = grpc.insecure_channel(addr)
    client = ServiceStub(channel)

    # ==============================================================
    # STEP 1: CREATE SESSION
    # ==============================================================
    new_resp = client.NewSession(SessionReq(srid=0))  # Euclidean coordinates
    if new_resp.id is None:
        raise RuntimeError("Server returned empty session id")
    session_id = new_resp.id.value
    print(f"Session created: {session_id}")

    # ==============================================================
    # STEP 2: PUSH GRID CELLS
    # ==============================================================
    # Road layout:
    #        V1 (vertical 1)    V2 (vertical 2)
    #          |                 |
    #    H ----+-----------------+---- H (horizontal)
    #          |                 |
    # Horizontal road cells: 0-9 (y=3.5, x=0..9)
    # Vertical road 1 cells: 10-19 (y=0..9, x=3.5)
    # Vertical road 2 cells: 20-29 (y=0..9, x=6.5)

    cells: list[Cell] = []
    cell_data: list[CellData] = []
    cell_coords: dict[int, tuple[float, float]] = {}

    # HORIZONTAL ROAD (cells 0-9)
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

    # VERTICAL ROAD 1 (cells 10-19, x=3.5)
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

    # VERTICAL ROAD 2 (cells 20-29, x=6.5)
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

    # Push grid via streaming
    def grid_requests() -> Iterator[SessionGrid]:
        yield SessionGrid(
            session_id=UUIDv4(value=session_id),
            data=cells,
        )

    for resp in client.PushSessionGrid(grid_requests()):
        print(f"Grid push response: code={resp.code} text={resp.text}")

    # ==============================================================
    # STEP 3: PUSH CONFLICT ZONES
    # ==============================================================
    conflict_zones = [
        ConflictZone(
            id=1,
            source_x=3,   # H cell before intersection
            target_x=4,   # H cell after intersection
            source_y=13,  # V1 cell before intersection
            target_y=14,  # V1 cell after intersection
            conflict_winner=ConflictWinnerType.CONFLICT_WINNER_SECOND,  # V1 priority
            conflict_type=ConflictZoneType.CONFLICT_ZONE_TYPE_UNDEFINED,
        ),
    ]

    def cz_requests() -> Iterator[SessionConflictZones]:
        yield SessionConflictZones(
            session_id=UUIDv4(value=session_id),
            data=conflict_zones,
        )

    for resp in client.PushSessionConflictZones(cz_requests()):
        print(f"Conflict zones push response: code={resp.code} text={resp.text}")

    # ==============================================================
    # STEP 4: PUSH TRAFFIC LIGHTS
    # ==============================================================
    traffic_lights = [
        TrafficLight(
            id=1,
            geom=Point(x=7.0, y=4.0),
            groups=[
                Group(
                    id=100,
                    label="Group block H",
                    cells=[6],
                    signals=["g", "r"],  # Green, Red
                    type=GroupType.GROUP_TYPE_VEHICLE,
                    crosswalk_length=0.0,
                ),
                Group(
                    id=200,
                    label="Group block V2",
                    cells=[23],
                    signals=["r", "g"],  # Red, Green
                    type=GroupType.GROUP_TYPE_VEHICLE,
                    crosswalk_length=0.0,
                ),
            ],
            times=[5, 5],  # 5s green, 5s red
        ),
    ]

    # Store TLS group cells for output later
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
        print(f"TLS push response: code={resp.code} text={resp.text}")

    # ==============================================================
    # STEP 5: PUSH TRIPS (vehicle generators)
    # ==============================================================
    trips = [
        Trip(
            id=1,
            trip_type=TripType.TRIP_TYPE_RANDOM,
            from_node=1,  # H birth
            to_node=9,    # H death
            initial_speed=1,
            probability=0.2,
            agent_type=AgentType.AGENT_TYPE_CAR,
            behaviour_type=BehaviourType.BEHAVIOUR_TYPE_COOPERATIVE,
        ),
        Trip(
            id=2,
            trip_type=TripType.TRIP_TYPE_RANDOM,
            from_node=10,  # V1 birth
            to_node=19,    # V1 death
            initial_speed=1,
            probability=0.3,
            agent_type=AgentType.AGENT_TYPE_CAR,
            behaviour_type=BehaviourType.BEHAVIOUR_TYPE_COOPERATIVE,
        ),
        Trip(
            id=3,
            trip_type=TripType.TRIP_TYPE_RANDOM,
            from_node=20,  # V2 birth
            to_node=29,    # V2 death
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
        print(f"Trip push response: code={resp.code} text={resp.text}")

    # ==============================================================
    # STEP 6: PRINT GRID/TLS METADATA FOR GNUPLOT
    # ==============================================================

    # Print TLS positions
    print("tl_id;x;y")
    for tl in traffic_lights:
        if tl.geom is not None:
            print(f"{tl.id};{tl.geom.x:.5f};{tl.geom.y:.5f}")

    # Print TLS controlled cells
    print("tl_id;controlled_cell;x;y")
    for tl in traffic_lights:
        for group in tl.groups:
            for cell_id in group.cells:
                if cell_id in cell_coords:
                    x, y = cell_coords[cell_id]
                    print(f"{tl.id};{cell_id};{x:.5f};{y:.5f}")

    # Print grid cells with connections
    print("cell_id;x;y;forward_x;forward_y;connection_type;zone")
    # First print all cells
    for cd in cell_data:
        print(f"{cd.id};{cd.x:.5f};{cd.y:.5f};{cd.x:.5f};{cd.y:.5f};cell;{zone_type_str(cd.zone_type)}")
    # Then print connections (arrows)
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

    # ==============================================================
    # STEP 7: RUN SIMULATION
    # ==============================================================
    steps_num = 50
    print(f"\n=== Running {steps_num} simulation steps ===\n")

    vehicle_states: list[VehicleStateRecord] = []
    tls_states: list[TLSStateRecord] = []

    def step_requests() -> Iterator[SessionStep]:
        for _ in range(steps_num):
            yield SessionStep(session_id=UUIDv4(value=session_id))

    for resp in client.SimulationStepSession(step_requests()):
        timestamp = resp.timestamp

        # Collect vehicle states
        for v in resp.vehicle_data:
            x = v.point.x if v.point else math.nan
            y = v.point.y if v.point else math.nan

            intermediate_cells = ",".join(str(c) for c in v.intermediate_cells)
            tail_cells = ",".join(str(c) for c in v.tail_cells)

            vehicle_states.append(VehicleStateRecord(
                step=timestamp,
                vehicle_id=v.vehicle_id,
                vehicle_type=agent_type_str(v.vehicle_type),
                speed=v.speed,
                bearing=v.bearing,
                intermediate_cells=intermediate_cells,
                cell=v.cell,
                x=x,
                y=y,
                tail_cells=tail_cells,
                trip_id=v.trip_id,
            ))

        # Collect TLS states (expand to per-cell)
        for tls_state in resp.tls_data:
            for group in tls_state.groups:
                key = (tls_state.id, group.id)
                if key in tls_group_cells:
                    for cell_id in tls_group_cells[key]:
                        x, y = cell_coords.get(cell_id, (0.0, 0.0))
                        tls_states.append(TLSStateRecord(
                            step=timestamp,
                            tl_id=tls_state.id,
                            group_id=group.id,
                            cell_id=cell_id,
                            x=x,
                            y=y,
                            signal=group.signal,
                        ))

    # Print vehicle states
    print("step;vehicle_id;vehicle_type;speed;bearing;intermediate_cells;cell;x;y;tail_cells;trip_id")
    for vs in vehicle_states:
        print(f"{vs.step};{vs.vehicle_id};{vs.vehicle_type};{vs.speed};{vs.bearing:.2f};"
              f"{vs.intermediate_cells};{vs.cell};{vs.x:.2f};{vs.y:.2f};{vs.tail_cells};{vs.trip_id}")

    # Print TLS states
    print("tl_step;tl_id;group_id;cell_id;x;y;signal")
    for ts in tls_states:
        print(f"{ts.step};{ts.tl_id};{ts.group_id};{ts.cell_id};{ts.x:.5f};{ts.y:.5f};{ts.signal}")

    print("\nSimulation complete!")


if __name__ == "__main__":
    main()
