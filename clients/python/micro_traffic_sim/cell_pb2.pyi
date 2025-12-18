import uuid_pb2 as _uuid_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ZoneType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ZONE_TYPE_UNDEFINED: _ClassVar[ZoneType]
    ZONE_TYPE_BIRTH: _ClassVar[ZoneType]
    ZONE_TYPE_DEATH: _ClassVar[ZoneType]
    ZONE_TYPE_COORDINATION: _ClassVar[ZoneType]
    ZONE_TYPE_COMMON: _ClassVar[ZoneType]
    ZONE_TYPE_ISOLATED: _ClassVar[ZoneType]
    ZONE_TYPE_LANE_FOR_BUS: _ClassVar[ZoneType]
    ZONE_TYPE_TRANSIT: _ClassVar[ZoneType]
    ZONE_TYPE_CROSSWALK: _ClassVar[ZoneType]
ZONE_TYPE_UNDEFINED: ZoneType
ZONE_TYPE_BIRTH: ZoneType
ZONE_TYPE_DEATH: ZoneType
ZONE_TYPE_COORDINATION: ZoneType
ZONE_TYPE_COMMON: ZoneType
ZONE_TYPE_ISOLATED: ZoneType
ZONE_TYPE_LANE_FOR_BUS: ZoneType
ZONE_TYPE_TRANSIT: ZoneType
ZONE_TYPE_CROSSWALK: ZoneType

class Cell(_message.Message):
    __slots__ = ("id", "geom", "zone_type", "speed_limit", "left_node", "forward_node", "right_node", "meso_link_id")
    ID_FIELD_NUMBER: _ClassVar[int]
    GEOM_FIELD_NUMBER: _ClassVar[int]
    ZONE_TYPE_FIELD_NUMBER: _ClassVar[int]
    SPEED_LIMIT_FIELD_NUMBER: _ClassVar[int]
    LEFT_NODE_FIELD_NUMBER: _ClassVar[int]
    FORWARD_NODE_FIELD_NUMBER: _ClassVar[int]
    RIGHT_NODE_FIELD_NUMBER: _ClassVar[int]
    MESO_LINK_ID_FIELD_NUMBER: _ClassVar[int]
    id: int
    geom: Point
    zone_type: ZoneType
    speed_limit: int
    left_node: int
    forward_node: int
    right_node: int
    meso_link_id: int
    def __init__(self, id: _Optional[int] = ..., geom: _Optional[_Union[Point, _Mapping]] = ..., zone_type: _Optional[_Union[ZoneType, str]] = ..., speed_limit: _Optional[int] = ..., left_node: _Optional[int] = ..., forward_node: _Optional[int] = ..., right_node: _Optional[int] = ..., meso_link_id: _Optional[int] = ...) -> None: ...

class Point(_message.Message):
    __slots__ = ("x", "y")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    x: float
    y: float
    def __init__(self, x: _Optional[float] = ..., y: _Optional[float] = ...) -> None: ...

class SessionGrid(_message.Message):
    __slots__ = ("session_id", "data")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    data: _containers.RepeatedCompositeFieldContainer[Cell]
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ..., data: _Optional[_Iterable[_Union[Cell, _Mapping]]] = ...) -> None: ...

class SessionGridResponse(_message.Message):
    __slots__ = ("code", "text")
    CODE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    code: int
    text: str
    def __init__(self, code: _Optional[int] = ..., text: _Optional[str] = ...) -> None: ...
