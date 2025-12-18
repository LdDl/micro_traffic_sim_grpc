import uuid_pb2 as _uuid_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ConflictWinnerType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CONFLICT_WINNER_UNDEFINED: _ClassVar[ConflictWinnerType]
    CONFLICT_WINNER_EQUAL: _ClassVar[ConflictWinnerType]
    CONFLICT_WINNER_FIRST: _ClassVar[ConflictWinnerType]
    CONFLICT_WINNER_SECOND: _ClassVar[ConflictWinnerType]

class ConflictZoneType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CONFLICT_ZONE_TYPE_UNDEFINED: _ClassVar[ConflictZoneType]
CONFLICT_WINNER_UNDEFINED: ConflictWinnerType
CONFLICT_WINNER_EQUAL: ConflictWinnerType
CONFLICT_WINNER_FIRST: ConflictWinnerType
CONFLICT_WINNER_SECOND: ConflictWinnerType
CONFLICT_ZONE_TYPE_UNDEFINED: ConflictZoneType

class ConflictZone(_message.Message):
    __slots__ = ("id", "source_x", "source_y", "target_x", "target_y", "conflict_winner", "conflict_type")
    ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_X_FIELD_NUMBER: _ClassVar[int]
    SOURCE_Y_FIELD_NUMBER: _ClassVar[int]
    TARGET_X_FIELD_NUMBER: _ClassVar[int]
    TARGET_Y_FIELD_NUMBER: _ClassVar[int]
    CONFLICT_WINNER_FIELD_NUMBER: _ClassVar[int]
    CONFLICT_TYPE_FIELD_NUMBER: _ClassVar[int]
    id: int
    source_x: int
    source_y: int
    target_x: int
    target_y: int
    conflict_winner: ConflictWinnerType
    conflict_type: ConflictZoneType
    def __init__(self, id: _Optional[int] = ..., source_x: _Optional[int] = ..., source_y: _Optional[int] = ..., target_x: _Optional[int] = ..., target_y: _Optional[int] = ..., conflict_winner: _Optional[_Union[ConflictWinnerType, str]] = ..., conflict_type: _Optional[_Union[ConflictZoneType, str]] = ...) -> None: ...

class SessionConflictZones(_message.Message):
    __slots__ = ("session_id", "data")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    data: _containers.RepeatedCompositeFieldContainer[ConflictZone]
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ..., data: _Optional[_Iterable[_Union[ConflictZone, _Mapping]]] = ...) -> None: ...

class SessionConflictZonesResponse(_message.Message):
    __slots__ = ("code", "text")
    CODE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    code: int
    text: str
    def __init__(self, code: _Optional[int] = ..., text: _Optional[str] = ...) -> None: ...
