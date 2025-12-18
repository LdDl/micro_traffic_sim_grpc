import uuid_pb2 as _uuid_pb2
import cell_pb2 as _cell_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SignalKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    SIGNAL_MAIN: _ClassVar[SignalKind]
    SIGNAL_INTERMEDIATE: _ClassVar[SignalKind]

class GroupType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    GROUP_TYPE_UNKNOWN: _ClassVar[GroupType]
    GROUP_TYPE_VEHICLE: _ClassVar[GroupType]
    GROUP_TYPE_PEDESTRIAN: _ClassVar[GroupType]

class MovementDirection(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    MOVEMENT_DIRECTION_UNKNOWN: _ClassVar[MovementDirection]
    MOVEMENT_DIRECTION_LEFT: _ClassVar[MovementDirection]
    MOVEMENT_DIRECTION_FORWARD: _ClassVar[MovementDirection]
    MOVEMENT_DIRECTION_RIGHT: _ClassVar[MovementDirection]
SIGNAL_MAIN: SignalKind
SIGNAL_INTERMEDIATE: SignalKind
GROUP_TYPE_UNKNOWN: GroupType
GROUP_TYPE_VEHICLE: GroupType
GROUP_TYPE_PEDESTRIAN: GroupType
MOVEMENT_DIRECTION_UNKNOWN: MovementDirection
MOVEMENT_DIRECTION_LEFT: MovementDirection
MOVEMENT_DIRECTION_FORWARD: MovementDirection
MOVEMENT_DIRECTION_RIGHT: MovementDirection

class TrafficLight(_message.Message):
    __slots__ = ("id", "geom", "groups", "times", "signals_kinds")
    ID_FIELD_NUMBER: _ClassVar[int]
    GEOM_FIELD_NUMBER: _ClassVar[int]
    GROUPS_FIELD_NUMBER: _ClassVar[int]
    TIMES_FIELD_NUMBER: _ClassVar[int]
    SIGNALS_KINDS_FIELD_NUMBER: _ClassVar[int]
    id: int
    geom: _cell_pb2.Point
    groups: _containers.RepeatedCompositeFieldContainer[Group]
    times: _containers.RepeatedScalarFieldContainer[int]
    signals_kinds: _containers.RepeatedScalarFieldContainer[SignalKind]
    def __init__(self, id: _Optional[int] = ..., geom: _Optional[_Union[_cell_pb2.Point, _Mapping]] = ..., groups: _Optional[_Iterable[_Union[Group, _Mapping]]] = ..., times: _Optional[_Iterable[int]] = ..., signals_kinds: _Optional[_Iterable[_Union[SignalKind, str]]] = ...) -> None: ...

class Group(_message.Message):
    __slots__ = ("id", "label", "geom", "cells", "signals", "movements", "crosswalk_length", "type")
    ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    GEOM_FIELD_NUMBER: _ClassVar[int]
    CELLS_FIELD_NUMBER: _ClassVar[int]
    SIGNALS_FIELD_NUMBER: _ClassVar[int]
    MOVEMENTS_FIELD_NUMBER: _ClassVar[int]
    CROSSWALK_LENGTH_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    id: int
    label: str
    geom: _containers.RepeatedCompositeFieldContainer[_cell_pb2.Point]
    cells: _containers.RepeatedScalarFieldContainer[int]
    signals: _containers.RepeatedScalarFieldContainer[str]
    movements: _containers.RepeatedCompositeFieldContainer[GroupMovementMetadata]
    crosswalk_length: float
    type: GroupType
    def __init__(self, id: _Optional[int] = ..., label: _Optional[str] = ..., geom: _Optional[_Iterable[_Union[_cell_pb2.Point, _Mapping]]] = ..., cells: _Optional[_Iterable[int]] = ..., signals: _Optional[_Iterable[str]] = ..., movements: _Optional[_Iterable[_Union[GroupMovementMetadata, _Mapping]]] = ..., crosswalk_length: _Optional[float] = ..., type: _Optional[_Union[GroupType, str]] = ...) -> None: ...

class GroupMovementMetadata(_message.Message):
    __slots__ = ("source", "target", "direction", "flow", "turn_radius")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    DIRECTION_FIELD_NUMBER: _ClassVar[int]
    FLOW_FIELD_NUMBER: _ClassVar[int]
    TURN_RADIUS_FIELD_NUMBER: _ClassVar[int]
    source: int
    target: int
    direction: MovementDirection
    flow: float
    turn_radius: float
    def __init__(self, source: _Optional[int] = ..., target: _Optional[int] = ..., direction: _Optional[_Union[MovementDirection, str]] = ..., flow: _Optional[float] = ..., turn_radius: _Optional[float] = ...) -> None: ...

class SessionTLS(_message.Message):
    __slots__ = ("session_id", "data")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    data: _containers.RepeatedCompositeFieldContainer[TrafficLight]
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ..., data: _Optional[_Iterable[_Union[TrafficLight, _Mapping]]] = ...) -> None: ...

class SessionTLSResponse(_message.Message):
    __slots__ = ("code", "text")
    CODE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    code: int
    text: str
    def __init__(self, code: _Optional[int] = ..., text: _Optional[str] = ...) -> None: ...
