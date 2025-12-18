import uuid_pb2 as _uuid_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SessionReq(_message.Message):
    __slots__ = ("srid",)
    SRID_FIELD_NUMBER: _ClassVar[int]
    srid: int
    def __init__(self, srid: _Optional[int] = ...) -> None: ...

class Session(_message.Message):
    __slots__ = ("id",)
    ID_FIELD_NUMBER: _ClassVar[int]
    id: _uuid_pb2.UUIDv4
    def __init__(self, id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ...) -> None: ...

class NewSessionResponse(_message.Message):
    __slots__ = ("code", "text", "id")
    CODE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    code: int
    text: str
    id: _uuid_pb2.UUIDv4
    def __init__(self, code: _Optional[int] = ..., text: _Optional[str] = ..., id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ...) -> None: ...

class InfoSessionResponse(_message.Message):
    __slots__ = ("code", "text", "data")
    CODE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    code: int
    text: str
    data: Session
    def __init__(self, code: _Optional[int] = ..., text: _Optional[str] = ..., data: _Optional[_Union[Session, _Mapping]] = ...) -> None: ...
