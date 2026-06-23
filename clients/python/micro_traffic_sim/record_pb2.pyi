import uuid_pb2 as _uuid_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RecordingState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    RECORDING_STATE_UNSPECIFIED: _ClassVar[RecordingState]
    RECORDING_STATE_NOT_RUNNING: _ClassVar[RecordingState]
    RECORDING_STATE_RUNNING: _ClassVar[RecordingState]
RECORDING_STATE_UNSPECIFIED: RecordingState
RECORDING_STATE_NOT_RUNNING: RecordingState
RECORDING_STATE_RUNNING: RecordingState

class RunAndRecordRequest(_message.Message):
    __slots__ = ("session_id", "horizon_ticks", "batch_ticks", "filter")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    HORIZON_TICKS_FIELD_NUMBER: _ClassVar[int]
    BATCH_TICKS_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    horizon_ticks: int
    batch_ticks: int
    filter: RecordFilter
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ..., horizon_ticks: _Optional[int] = ..., batch_ticks: _Optional[int] = ..., filter: _Optional[_Union[RecordFilter, _Mapping]] = ...) -> None: ...

class RecordFilter(_message.Message):
    __slots__ = ("sample_period", "meso_link_ids")
    SAMPLE_PERIOD_FIELD_NUMBER: _ClassVar[int]
    MESO_LINK_IDS_FIELD_NUMBER: _ClassVar[int]
    sample_period: int
    meso_link_ids: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, sample_period: _Optional[int] = ..., meso_link_ids: _Optional[_Iterable[int]] = ...) -> None: ...

class RunAndRecordResponse(_message.Message):
    __slots__ = ("metadata", "batch", "summary")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    BATCH_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    metadata: RunMetadata
    batch: RecordBatch
    summary: RunSummary
    def __init__(self, metadata: _Optional[_Union[RunMetadata, _Mapping]] = ..., batch: _Optional[_Union[RecordBatch, _Mapping]] = ..., summary: _Optional[_Union[RunSummary, _Mapping]] = ...) -> None: ...

class RunMetadata(_message.Message):
    __slots__ = ("format_version", "tick_seconds", "spawn_seed", "stochastic_seed", "core_version", "rand_version", "config_hash", "schema", "tl_schema")
    FORMAT_VERSION_FIELD_NUMBER: _ClassVar[int]
    TICK_SECONDS_FIELD_NUMBER: _ClassVar[int]
    SPAWN_SEED_FIELD_NUMBER: _ClassVar[int]
    STOCHASTIC_SEED_FIELD_NUMBER: _ClassVar[int]
    CORE_VERSION_FIELD_NUMBER: _ClassVar[int]
    RAND_VERSION_FIELD_NUMBER: _ClassVar[int]
    CONFIG_HASH_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    TL_SCHEMA_FIELD_NUMBER: _ClassVar[int]
    format_version: int
    tick_seconds: float
    spawn_seed: int
    stochastic_seed: int
    core_version: str
    rand_version: str
    config_hash: str
    schema: ColumnSchema
    tl_schema: ColumnSchema
    def __init__(self, format_version: _Optional[int] = ..., tick_seconds: _Optional[float] = ..., spawn_seed: _Optional[int] = ..., stochastic_seed: _Optional[int] = ..., core_version: _Optional[str] = ..., rand_version: _Optional[str] = ..., config_hash: _Optional[str] = ..., schema: _Optional[_Union[ColumnSchema, _Mapping]] = ..., tl_schema: _Optional[_Union[ColumnSchema, _Mapping]] = ...) -> None: ...

class ColumnSchema(_message.Message):
    __slots__ = ("columns",)
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    columns: _containers.RepeatedCompositeFieldContainer[ColumnDef]
    def __init__(self, columns: _Optional[_Iterable[_Union[ColumnDef, _Mapping]]] = ...) -> None: ...

class ColumnDef(_message.Message):
    __slots__ = ("name", "type")
    NAME_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    name: str
    type: str
    def __init__(self, name: _Optional[str] = ..., type: _Optional[str] = ...) -> None: ...

class RecordBatch(_message.Message):
    __slots__ = ("tick_start", "tick_count", "total_rows", "columns")
    TICK_START_FIELD_NUMBER: _ClassVar[int]
    TICK_COUNT_FIELD_NUMBER: _ClassVar[int]
    TOTAL_ROWS_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    tick_start: int
    tick_count: int
    total_rows: int
    columns: bytes
    def __init__(self, tick_start: _Optional[int] = ..., tick_count: _Optional[int] = ..., total_rows: _Optional[int] = ..., columns: _Optional[bytes] = ...) -> None: ...

class RunSummary(_message.Message):
    __slots__ = ("total_ticks", "total_rows", "total_bytes", "vehicles_completed", "vehicles_lost")
    TOTAL_TICKS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_ROWS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_BYTES_FIELD_NUMBER: _ClassVar[int]
    VEHICLES_COMPLETED_FIELD_NUMBER: _ClassVar[int]
    VEHICLES_LOST_FIELD_NUMBER: _ClassVar[int]
    total_ticks: int
    total_rows: int
    total_bytes: int
    vehicles_completed: int
    vehicles_lost: int
    def __init__(self, total_ticks: _Optional[int] = ..., total_rows: _Optional[int] = ..., total_bytes: _Optional[int] = ..., vehicles_completed: _Optional[int] = ..., vehicles_lost: _Optional[int] = ...) -> None: ...

class RecordingStatusRequest(_message.Message):
    __slots__ = ("session_id",)
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ...) -> None: ...

class RecordingStatusResponse(_message.Message):
    __slots__ = ("state", "current_tick", "rows", "cancel_requested")
    STATE_FIELD_NUMBER: _ClassVar[int]
    CURRENT_TICK_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    CANCEL_REQUESTED_FIELD_NUMBER: _ClassVar[int]
    state: RecordingState
    current_tick: int
    rows: int
    cancel_requested: bool
    def __init__(self, state: _Optional[_Union[RecordingState, str]] = ..., current_tick: _Optional[int] = ..., rows: _Optional[int] = ..., cancel_requested: bool = ...) -> None: ...

class StopRecordingRequest(_message.Message):
    __slots__ = ("session_id",)
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: _uuid_pb2.UUIDv4
    def __init__(self, session_id: _Optional[_Union[_uuid_pb2.UUIDv4, _Mapping]] = ...) -> None: ...

class StopRecordingResponse(_message.Message):
    __slots__ = ("accepted",)
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    accepted: bool
    def __init__(self, accepted: bool = ...) -> None: ...
