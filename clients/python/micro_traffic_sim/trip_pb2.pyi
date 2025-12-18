import uuid_pb2 as _uuid_pb2
import step_pb2 as _step_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class TripType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TRIP_TYPE_UNDEFINED: _ClassVar[TripType]
    TRIP_TYPE_CONSTANT: _ClassVar[TripType]
    TRIP_TYPE_RANDOM: _ClassVar[TripType]

class BehaviourType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    BEHAVIOUR_TYPE_UNDEFINED: _ClassVar[BehaviourType]
    BEHAVIOUR_TYPE_BLOCK: _ClassVar[BehaviourType]
    BEHAVIOUR_TYPE_AGGRESSIVE: _ClassVar[BehaviourType]
    BEHAVIOUR_TYPE_COOPERATIVE: _ClassVar[BehaviourType]
    BEHAVIOUR_TYPE_LIMIT_SPEED_BY_TRIP: _ClassVar[BehaviourType]
TRIP_TYPE_UNDEFINED: TripType
TRIP_TYPE_CONSTANT: TripType
TRIP_TYPE_RANDOM: TripType
BEHAVIOUR_TYPE_UNDEFINED: BehaviourType
BEHAVIOUR_TYPE_BLOCK: BehaviourType
BEHAVIOUR_TYPE_AGGRESSIVE: BehaviourType
BEHAVIOUR_TYPE_COOPERATIVE: BehaviourType
BEHAVIOUR_TYPE_LIMIT_SPEED_BY_TRIP: BehaviourType

class Trip(_message.Message):
    __slots__ = ("id", "trip_type", "from_node", "to_node", "initial_speed", "probability", "agent_type", "behaviour_type", "time", "start_time", "end_time", "relax_time", "transits")
    ID_FIELD_NUMBER: _ClassVar[int]
    TRIP_TYPE_FIELD_NUMBER: _ClassVar[int]
    FROM_NODE_FIELD_NUMBER: _ClassVar[int]
    TO_NODE_FIELD_NUMBER: _ClassVar[int]
    INITIAL_SPEED_FIELD_NUMBER: _ClassVar[int]
    PROBABILITY_FIELD_NUMBER: _ClassVar[int]
    AGENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    BEHAVIOUR_TYPE_FIELD_NUMBER: _ClassVar[int]
    TIME_FIELD_NUMBER: _ClassVar[int]
    START_TIME_FIELD_NUMBER: _ClassVar[int]
    END_TIME_FIELD_NUMBER: _ClassVar[int]
    RELAX_TIME_FIELD_NUMBER: _ClassVar[int]
    TRANSITS_FIELD_NUMBER: _ClassVar[int]
    id: int
    trip_type: TripType
    from_node: int
    to_node: int
    initial_speed: int
    probability: float
    agent_type: _step_pb2.AgentType
    behaviour_type: BehaviourType
    time: int
    start_time: int
    end_time: int
    relax_time: int
    transits: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, id: _Optional[int] = ..., trip_type: _Optional[_Union[TripType, str]] = ..., from_node: _Optional[int] = ..., to_node: _Optional[int] = ..., initial_speed: _Optional[int] = ..., probability: _Optional[float] = ..., agent_type: _Optional[_Union[_step_pb2.AgentType, str]] = ..., behaviour_type: _Optional[_Union[BehaviourType, str]] = ..., time: _Optional[int] = ..., start_time: _Optional[int] = ..., end_time: _Optional[int] = ..., relax_time: _Optional[int] = ..., transits: _Optional[_Iterable[int]] = ...) -> None: ...

class SessionTrip(_message.Message):
    __slots__ = ("session_id", "data")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    data: _containers.RepeatedCompositeFieldContainer[Trip]
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ..., data: _Optional[_Iterable[_Union[Trip, _Mapping]]] = ...) -> None: ...

class SessionTripResponse(_message.Message):
    __slots__ = ("code", "text")
    CODE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    code: int
    text: str
    def __init__(self, code: _Optional[int] = ..., text: _Optional[str] = ...) -> None: ...
