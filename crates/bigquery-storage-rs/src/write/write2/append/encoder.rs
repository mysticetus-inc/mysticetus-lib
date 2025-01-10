use std::marker::PhantomData;
use std::sync::RwLockReadGuard;

use bytes::{Bytes, BytesMut};
use protos::bigquery_storage::append_rows_request::{ProtoData, Rows};
use protos::bigquery_storage::{AppendRowsRequest, ProtoRows, ProtoSchema};

use super::super::{StreamType, WriteSession};
use super::state::RequestState;
use crate::proto::ProtoSerializer;
use crate::write::Schema;
use crate::write::write2::WriteSessionState;

// The max size for 1 appendrowsRequest. The actual limit is 10MiB but due to
// expensive to compute encoded sizes for the message, we do simpler checks but
// limit to 9MiB instead (giving another MiB buffer for slightly off approximations)
const MAX_REQUEST_SIZE: usize = 9 * 1024 * 1024;

pub struct RowEncoder<R> {
    next_message: Option<NextMessage>,
    leftover_row: Option<Bytes>,
    max_rows_per_message: usize,
    max_encoded_row_len: usize,
    _marker: PhantomData<fn(R)>,
}

impl<R: serde::Serialize> RowEncoder<R> {
    pub(super) fn new<Type: StreamType>(
        write_session: &WriteSession<R, Type>,
        init_message: AppendRowsRequest,
    ) -> Self {
        let schema = write_session.state().schema().to_descriptor_proto();

        let proto_schema = ProtoSchema {
            proto_descriptor: Some(schema),
        };

        Self {
            next_message: Some(NextMessage::new(init_message, Some(proto_schema), 256)),
            leftover_row: None,
            max_encoded_row_len: 0,
            max_rows_per_message: 0,
            _marker: PhantomData,
        }
    }

    #[must_use = "the Some case needs to be handled"]
    pub fn finish<'a, Type: StreamType>(
        mut self,
        write_session: &'a WriteSession<R, Type>,
        guard: &mut Option<RwLockReadGuard<'a, WriteSessionState<Type>>>,
        state: RequestState,
    ) -> Option<super::EncodedRequest> {
        self.take_if_not_empty(write_session, guard, state)
            .map(super::EncodedRequest)
    }

    pub(super) fn take_if_not_empty<'a, Type: StreamType>(
        &mut self,
        write_session: &'a WriteSession<R, Type>,
        guard: &mut Option<RwLockReadGuard<'a, WriteSessionState<Type>>>,
        state: RequestState,
    ) -> Option<AppendRowsRequest> {
        match (self.next_message.take(), self.leftover_row.take()) {
            (Some(msg), None) => {
                // ignore if empty
                if msg.len() > 0 {
                    Some(msg.build())
                } else {
                    None
                }
            }
            (Some(mut msg), Some(leftover)) => {
                // try to append the leftover row, handling it in the next request if needed
                if let Err(leftover) = msg.append_encoded_row(leftover) {
                    self.leftover_row = Some(leftover);
                }
                Some(msg.build())
            }
            (None, Some(leftover)) => {
                let message = crate::write::write2::session::make_base_message(
                    write_session.shared(),
                    guard,
                    state,
                );

                let mut next = NextMessage::new(message, None, 1);
                next.append_encoded_row(leftover)
                    .expect("overflowed max request size with 1 row");

                Some(next.build())
            }
            (None, None) => None,
        }
    }

    pub(super) fn append_row<Type: StreamType>(
        &mut self,
        write_session: &WriteSession<R, Type>,
        schema: &Schema,
        state: RequestState,
        row: R,
    ) -> crate::Result<Option<AppendRowsRequest>> {
        fn encode_row<Row: serde::Serialize>(
            row: Row,
            est_row_size: usize,
            schema: &Schema,
        ) -> crate::Result<Bytes> {
            // use a default of 1KiB for the first row to avoid lots of resizing
            // when starting from empty
            let mut buf = BytesMut::with_capacity(if est_row_size == 0 {
                1024
            } else {
                est_row_size
            });

            ProtoSerializer::new(&mut buf, schema).serialize_row(&row)?;

            Ok(buf.freeze())
        }

        let encoded = encode_row(row, self.max_encoded_row_len, schema)?;

        self.max_encoded_row_len = self.max_encoded_row_len.max(encoded.len());

        let next_message = get_or_insert_default_message(
            &mut self.next_message,
            write_session,
            state,
            self.max_rows_per_message,
            &mut self.leftover_row,
        );

        match next_message.append_encoded_row(encoded) {
            Ok(()) => Ok(None),
            Err(encoded) => {
                self.leftover_row = Some(encoded);
                let next_message = self
                    .next_message
                    .take()
                    .expect("get_or_insert_default_message ensures this is some");

                self.max_rows_per_message = self.max_rows_per_message.max(next_message.len());

                Ok(Some(next_message.build()))
            }
        }
    }
}

struct NextMessage {
    base_message: AppendRowsRequest,
    data: ProtoData,
    current_size: usize,
}

impl NextMessage {
    fn new(
        base_message: AppendRowsRequest,
        writer_schema: Option<ProtoSchema>,
        est_rows: usize,
    ) -> Self {
        let data = ProtoData {
            writer_schema,
            rows: Some(ProtoRows {
                serialized_rows: Vec::with_capacity(est_rows),
            }),
        };

        let current_size =
            prost::Message::encoded_len(&base_message) + prost::Message::encoded_len(&data);

        Self {
            base_message,
            current_size,
            data,
        }
    }

    fn append_encoded_row(&mut self, row: Bytes) -> Result<(), Bytes> {
        let row_size = prost::length_delimiter_len(row.len()) + row.len();

        if self.current_size + row_size > MAX_REQUEST_SIZE {
            return Err(row);
        }

        self.current_size += row_size;

        self.data
            .rows
            .as_mut()
            .expect("NextMessage should initialize this")
            .serialized_rows
            .push(row);

        Ok(())
    }

    fn len(&self) -> usize {
        self.data
            .rows
            .as_ref()
            .map(|rows| rows.serialized_rows.len())
            .unwrap_or(0)
    }

    fn build(self) -> AppendRowsRequest {
        let Self {
            mut base_message,
            mut data,
            ..
        } = self;

        // try and free any memory we allocated too much of
        if let Some(rows) = data.rows.as_mut() {
            rows.serialized_rows.shrink_to_fit();
        }

        base_message.rows = Some(Rows::ProtoRows(data));

        base_message
    }
}

fn get_or_insert_default_message<'a, R, Type: StreamType>(
    opt: &'a mut Option<NextMessage>,
    write_session: &WriteSession<R, Type>,
    state: RequestState,
    est_row_count: usize,
    leftover_row: &mut Option<Bytes>,
) -> &'a mut NextMessage {
    opt.get_or_insert_with(|| {
        let mut next_msg =
            NextMessage::new(write_session.make_base_message(state), None, est_row_count);

        if let Some(leftover) = leftover_row.take() {
            next_msg
                .append_encoded_row(leftover)
                .expect("single row overflows the maximum request size");
        }

        next_msg
    })
}
