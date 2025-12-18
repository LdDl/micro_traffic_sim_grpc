"""
micro_traffic_sim - Python gRPC client for microscopic traffic simulation.

Usage:
    import grpc
    from micro_traffic_sim import service_pb2_grpc, session_pb2, cell_pb2

    channel = grpc.insecure_channel("127.0.0.1:50051")
    client = service_pb2_grpc.ServiceStub(channel)

    # Create session
    response = client.NewSession(session_pb2.SessionReq(srid=0))
    session_id = response.id.value
"""

from .uuid_pb2 import UUIDv4
from .cell_pb2 import Cell, Point, SessionGrid, SessionGridResponse, ZoneType
from .session_pb2 import SessionReq, NewSessionResponse, InfoSessionResponse
from .trip_pb2 import (
    Trip,
    SessionTrip,
    SessionTripResponse,
    TripType,
    BehaviourType,
)
from .tls_pb2 import (
    TrafficLight,
    Group,
    GroupType,
    SessionTLS,
    SessionTLSResponse,
)
from .conflict_zones_pb2 import (
    ConflictZone,
    ConflictWinnerType,
    ConflictZoneType,
    SessionConflictZones,
    SessionConflictZonesResponse,
)
from .step_pb2 import (
    SessionStep,
    SessionStepResponse,
    VehicleState,
    TLSState,
    TLGroup,
    AgentType,
)
from .service_pb2_grpc import ServiceStub

__all__ = [
    # UUID
    "UUIDv4",
    # Cell
    "Cell",
    "Point",
    "SessionGrid",
    "SessionGridResponse",
    "ZoneType",
    # Session
    "SessionReq",
    "NewSessionResponse",
    "InfoSessionResponse",
    # Trip
    "Trip",
    "SessionTrip",
    "SessionTripResponse",
    "TripType",
    "BehaviourType",
    # TLS
    "TrafficLight",
    "Group",
    "GroupType",
    "SessionTLS",
    "SessionTLSResponse",
    # Conflict Zones
    "ConflictZone",
    "ConflictWinnerType",
    "ConflictZoneType",
    "SessionConflictZones",
    "SessionConflictZonesResponse",
    # Step
    "SessionStep",
    "SessionStepResponse",
    "VehicleState",
    "TLSState",
    "TLGroup",
    "AgentType",
    # Service
    "ServiceStub",
]

__version__ = "0.0.1"
