use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use gcp_auth_channel::AuthChannel;
use net_utils::bidi2::{RequestSink, RequestStream};
use protos::bigquery_storage::append_rows_request::{ProtoData, Rows};
use protos::bigquery_storage::{AppendRowsRequest, AppendRowsResponse, ProtoSchema, WriteStream};
use protos::protobuf::Int64Value;

use super::append::RequestState;
use super::missing_value::MissingValueInterpretations;
use super::types::Boolean;
use super::{DefaultStream, Schema, StreamType};
use crate::{BigQueryStorageClient, Error};

pub struct WriteSession<R, Type: StreamType = DefaultStream> {
    pub(super) channel: AuthChannel,
    shared: Arc<WriteSessionShared<Type>>,
    _marker: PhantomData<fn(R)>,
}

pub struct WriteSessionShared<Type: StreamType> {
    state: RwLock<WriteSessionState<Type>>,
    enforce_offsets: bool,
    missing_values: Option<MissingValueInterpretations>,
    trace_id: Option<Box<str>>,
}

impl<Type: StreamType> WriteSessionShared<Type> {
    fn state(&self) -> RwLockReadGuard<'_, WriteSessionState<Type>> {
        self.state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub(super) fn make_base_message<'a, Type: StreamType>(
    shared: &'a WriteSessionShared<Type>,
    session_state_guard: &mut Option<RwLockReadGuard<'a, WriteSessionState<Type>>>,
    state: RequestState,
) -> AppendRowsRequest {
    let write_stream = if state.needs_name() {
        session_state_guard
            .get_or_insert_with(|| shared.state())
            .stream_type()
            .write_stream()
            .into_owned()
    } else {
        String::new()
    };

    let offset = if <Type::OffsetAllowed as Boolean>::VALUE && shared.enforce_offsets {
        session_state_guard
            .get_or_insert_with(|| shared.state())
            .stream_type()
            .offset()
    } else {
        None
    };

    let rows = state.needs_schema().then(|| {
        let writer_schema = session_state_guard
            .get_or_insert_with(|| shared.state())
            .writer_schema();

        Rows::ProtoRows(ProtoData {
            rows: None,
            writer_schema: Some(writer_schema),
        })
    });

    // get rid of a lock ASAP if we have it
    drop(state);

    let (missing, default) = MissingValueInterpretations::pair_from_opt(&shared.missing_values);

    let trace_id = shared
        .trace_id
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(String::new);

    AppendRowsRequest {
        write_stream,
        offset: offset.map(|value| Int64Value { value }),
        trace_id,
        missing_value_interpretations: missing,
        default_missing_value_interpretation: default as i32,
        rows,
    }
}

impl<R, Type: StreamType> WriteSession<R, Type> {
    pub(super) fn new(
        channel: AuthChannel,
        stream_type: Type,
        schema: Schema,
        missing_values: Option<MissingValueInterpretations>,
        enforce_offsets: bool,
        trace_id: Option<Box<str>>,
    ) -> Self {
        Self {
            channel,
            shared: Arc::new(WriteSessionShared {
                trace_id,
                missing_values,
                enforce_offsets,
                state: RwLock::new(WriteSessionState {
                    schema,
                    stream_type,
                }),
            }),
            _marker: PhantomData,
        }
    }

    pub(super) fn update_offset(&self, offset: i64) {
        if <Type::OffsetAllowed as Boolean>::VALUE {
            self.state_mut().stream_type_mut().update_offset(offset);
        }
    }

    pub(super) fn update_schema(&self, schema: Schema) {
        self.state_mut().schema = schema;
    }

    pub(super) fn shared(&self) -> &Arc<WriteSessionShared<Type>> {
        &self.shared
    }

    pub(super) fn with_schema<O>(&self, with_fn: impl FnOnce(&Schema) -> O) -> O {
        let state = self.state();
        with_fn(&state.schema)
    }

    pub(super) fn trace_id(&self) -> Option<&str> {
        self.shared.trace_id.as_deref()
    }

    pub(super) fn state(&self) -> RwLockReadGuard<'_, WriteSessionState<Type>> {
        self.shared
            .state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) fn state_mut(&self) -> RwLockWriteGuard<'_, WriteSessionState<Type>> {
        self.shared
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) fn stream_name(&self) -> String {
        self.state().stream_type.write_stream().into_owned()
    }

    pub(super) fn make_base_message(&self, request_state: RequestState) -> AppendRowsRequest {
        make_base_message(&self.shared, &mut None, request_state)
    }
}

impl<R, Type: StreamType> Clone for WriteSession<R, Type> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
            shared: Arc::clone(&self.shared),
            _marker: PhantomData,
        }
    }
}

pub(super) struct WriteSessionState<Type: StreamType> {
    schema: Schema,
    stream_type: Type,
}

impl<Type: StreamType> WriteSessionState<Type> {
    pub(super) fn writer_schema(&self) -> ProtoSchema {
        ProtoSchema {
            proto_descriptor: Some(self.schema.to_descriptor_proto()),
        }
    }

    #[inline]
    pub fn stream_type(&self) -> &Type {
        &self.stream_type
    }

    #[inline]
    pub fn stream_type_mut(&mut self) -> &mut Type {
        &mut self.stream_type
    }

    #[inline]
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

impl<R, Type: StreamType> WriteSession<R, Type> {}
