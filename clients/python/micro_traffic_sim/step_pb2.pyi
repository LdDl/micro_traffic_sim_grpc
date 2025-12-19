import uuid_pb2 as _uuid_pb2
import cell_pb2 as _cell_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class AgentType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    AGENT_TYPE_UNDEFINED: _ClassVar[AgentType]
    AGENT_TYPE_CAR: _ClassVar[AgentType]
    AGENT_TYPE_BUS: _ClassVar[AgentType]
    AGENT_TYPE_TAXI: _ClassVar[AgentType]
    AGENT_TYPE_PEDESTRIAN: _ClassVar[AgentType]
    AGENT_TYPE_TRUCK: _ClassVar[AgentType]
    AGENT_TYPE_LARGE_BUS: _ClassVar[AgentType]
AGENT_TYPE_UNDEFINED: AgentType
AGENT_TYPE_CAR: AgentType
AGENT_TYPE_BUS: AgentType
AGENT_TYPE_TAXI: AgentType
AGENT_TYPE_PEDESTRIAN: AgentType
AGENT_TYPE_TRUCK: AgentType
AGENT_TYPE_LARGE_BUS: AgentType

class SessionStep(_message.Message):
    __slots__ = ("session_id",)
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ...) -> None: ...

class SessionStepResponse(_message.Message):
    __slots__ = ("code", "text", "timestamp", "vehicle_data", "tls_data")
    CODE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    VEHICLE_DATA_FIELD_NUMBER: _ClassVar[int]
    TLS_DATA_FIELD_NUMBER: _ClassVar[int]
    code: int
    text: str
    timestamp: int
    vehicle_data: _containers.RepeatedCompositeFieldContainer[VehicleState]
    tls_data: _containers.RepeatedCompositeFieldContainer[TLSState]
    def __init__(self, code: _Optional[int] = ..., text: _Optional[str] = ..., timestamp: _Optional[int] = ..., vehicle_data: _Optional[_Iterable[_Union[VehicleState, _Mapping]]] = ..., tls_data: _Optional[_Iterable[_Union[TLSState, _Mapping]]] = ...) -> None: ...

class VehicleState(_message.Message):
    __slots__ = ("vehicle_id", "vehicle_type", "point", "bearing", "speed", "cell", "intermediate_cells", "travel_time", "trip_id", "tail_cells")
    VEHICLE_ID_FIELD_NUMBER: _ClassVar[int]
    VEHICLE_TYPE_FIELD_NUMBER: _ClassVar[int]
    POINT_FIELD_NUMBER: _ClassVar[int]
    BEARING_FIELD_NUMBER: _ClassVar[int]
    SPEED_FIELD_NUMBER: _ClassVar[int]
    CELL_FIELD_NUMBER: _ClassVar[int]
    INTERMEDIATE_CELLS_FIELD_NUMBER: _ClassVar[int]
    TRAVEL_TIME_FIELD_NUMBER: _ClassVar[int]
    TRIP_ID_FIELD_NUMBER: _ClassVar[int]
    TAIL_CELLS_FIELD_NUMBER: _ClassVar[int]
    vehicle_id: int
    vehicle_type: AgentType
    point: _cell_pb2.Point
    bearing: float
    speed: int
    cell: int
    intermediate_cells: _containers.RepeatedScalarFieldContainer[int]
    travel_time: int
    trip_id: int
    tail_cells: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, vehicle_id: _Optional[int] = ..., vehicle_type: _Optional[_Union[AgentType, str]] = ..., point: _Optional[_Union[_cell_pb2.Point, _Mapping]] = ..., bearing: _Optional[float] = ..., speed: _Optional[int] = ..., cell: _Optional[int] = ..., intermediate_cells: _Optional[_Iterable[int]] = ..., travel_time: _Optional[int] = ..., trip_id: _Optional[int] = ..., tail_cells: _Optional[_Iterable[int]] = ...) -> None: ...

class TLSState(_message.Message):
    __slots__ = ("id", "groups")
    ID_FIELD_NUMBER: _ClassVar[int]
    GROUPS_FIELD_NUMBER: _ClassVar[int]
    id: int
    groups: _containers.RepeatedCompositeFieldContainer[TLGroup]
    def __init__(self, id: _Optional[int] = ..., groups: _Optional[_Iterable[_Union[TLGroup, _Mapping]]] = ...) -> None: ...

class TLGroup(_message.Message):
    __slots__ = ("id", "signal")
    ID_FIELD_NUMBER: _ClassVar[int]
    SIGNAL_FIELD_NUMBER: _ClassVar[int]
    id: int
    signal: str
    def __init__(self, id: _Optional[int] = ..., signal: _Optional[str] = ...) -> None: ...
