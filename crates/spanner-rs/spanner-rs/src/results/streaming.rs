use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::Stream;
use protos::protobuf::value::Kind;
use protos::protobuf::{self, ListValue};
use protos::spanner::{self, PartialResultSet};

use super::iter::ResultIter;
use crate::results::FieldIndex;
use crate::table::Table;

pin_project_lite::pin_project! {
    pub struct StreamingRead<T: Table> {
        #[pin]
        streaming: tonic::Streaming<PartialResultSet>,
        field_index: Option<FieldIndex<T::NumColumns>>,
        chunked_last_value: Option<Kind>,
        existing_iter: Option<ResultIter<T>>,
        // PartialResultSets yield values, not rows, so we need to push them into a container
        // until we get N values, where N is the number of values in 'fields'
        partial_row: Vec<protobuf::Value>,
    }
}

impl<T: Table> StreamingRead<T> {
    pub async fn next_chunk(self: &mut Pin<&mut Self>) -> crate::Result<Option<ResultIter<T>>> {
        futures::future::poll_fn(move |cx| self.as_mut().poll_next(cx))
            .await
            .transpose()
    }

    pub async fn collect_to_vec(self: Pin<&mut Self>) -> crate::Result<Vec<T>> {
        let mut dst = Vec::new();
        self.collect_into(&mut dst).await?;
        Ok(dst)
    }

    #[inline]
    pub(crate) const fn from_streaming(streaming: tonic::Streaming<PartialResultSet>) -> Self {
        Self {
            streaming,
            field_index: None,
            chunked_last_value: None,
            existing_iter: None,
            partial_row: Vec::new(),
        }
    }

    pub async fn collect_into<C>(mut self: Pin<&mut Self>, dst: &mut C) -> crate::Result<usize>
    where
        C: Extend<T>,
    {
        let mut count = 0;

        while let Some(chunk) = self.next_chunk().await? {
            dst.extend_reserve(chunk.len());

            for res in chunk {
                let row = res?;
                dst.extend_one(row);
                count += 1;
            }
        }

        Ok(count)
    }

    pub(crate) fn new_with_first_chunk(
        streaming: tonic::Streaming<PartialResultSet>,
        mut first_chunk: PartialResultSet,
    ) -> crate::Result<Self> {
        let mut new = Self::from_streaming(streaming);

        // inserted at the end of the function, so we can use it while constructing 'new'.
        let field_index = FieldIndex::from_result_set_meta::<T>(first_chunk.metadata.take())?;

        if first_chunk.chunked_value {
            new.chunked_last_value = first_chunk.values.pop().and_then(|value| value.kind);
        }

        let full_rows = first_chunk.values.len() / field_index.len();
        let mut value_iter = first_chunk.values.into_iter();

        // if we can build a full row, assemble it as an existing iterator rather than
        // an oversize partial_row.
        if full_rows > 0 {
            let mut rows = Vec::with_capacity(full_rows);

            for _ in 0..full_rows {
                let mut values = Vec::with_capacity(field_index.len());
                values.extend(value_iter.by_ref().take(field_index.len()));
                rows.push(ListValue { values });
            }

            new.existing_iter = Some(ResultIter::from_parts(
                field_index.clone(),
                rows,
                first_chunk.stats,
            ));
        }

        // if we any elements left over that couldn't populate a full row,
        // we need to put them in the partial_row, or we'll lose them.
        if !value_iter.is_empty() {
            new.partial_row.reserve(field_index.len());
            new.partial_row.extend(value_iter);
        }

        // insert this here, so we can use it for length calculations in the above code.
        new.field_index = Some(field_index);

        Ok(new)
    }

    pub(crate) async fn new_extract_tx(
        mut streaming: tonic::Streaming<PartialResultSet>,
    ) -> crate::Result<(spanner::Transaction, Self)> {
        let mut message = match streaming.message().await? {
            Some(msg) => msg,
            None => {
                return Err(crate::Error::Misc(anyhow::anyhow!(
                    "PartialResultSet stream empty, expected at least 1"
                )));
            }
        };

        let tx = message
            .metadata
            .as_mut()
            .and_then(|meta| meta.transaction.take())
            .ok_or_else(|| anyhow::anyhow!("Expected newly begun transaction in ResultSet"))?;

        let new = Self::new_with_first_chunk(streaming, message)?;

        Ok((tx, new))
    }
}

impl<T: Table> Stream for StreamingRead<T> {
    type Item = crate::Result<ResultIter<T>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // run this in a loop, just in case we get a PartialResultSet that doesn't fill up an entire
        // row. that way we poll the stream again, and make sure an updated waker is
        // registered.
        loop {
            let mut chunk = match ready!(this.streaming.as_mut().poll_next(cx)) {
                Some(Ok(chunk)) => chunk,
                Some(Err(error)) => return Poll::Ready(Some(Err(crate::Error::from(error)))),
                None => return Poll::Ready(None),
            };

            // first partial result set is supposed to populate this, remaining messages won't have
            // metadata.
            if this.field_index.is_none() {
                match chunk.metadata.take() {
                    Some(spanner::ResultSetMetadata {
                        row_type: Some(struct_type),
                        ..
                    }) => match FieldIndex::from_struct_type::<T>(struct_type) {
                        Ok(field_index) => *this.field_index = Some(field_index),
                        Err(error) => return Poll::Ready(Some(Err(error.into()))),
                    },
                    _ => return Poll::Ready(Some(Err(crate::Error::MissingResultMetadata))),
                }
            }

            // merge any previous chunked value
            if let Some(chunked_value) = this.chunked_last_value.take() {
                // try and do this in-place. This saves us from having to 2
                // O(n) operations (remove + insert).
                match chunk.values.first_mut() {
                    Some(protobuf::Value { kind: Some(parent) }) => {
                        if let Err(new) = chunked::merge_value_kind(parent, chunked_value) {
                            panic!(
                                "invalid chunked value:\n\nparent {parent:#?}\n\nchunk {new:#?}"
                            );
                        }
                    }
                    // TODO: empty values straight should likely be an error case.
                    // for now, just fill in the empty value with the chunk.
                    Some(empty_value @ protobuf::Value { kind: None }) => {
                        empty_value.kind = Some(chunked_value);
                    }
                    None => todo!("not 100% sure how to handle this case yet"),
                };
            }

            if chunk.chunked_value {
                // make sure we arent clobbering an old value that didnt get merged
                debug_assert!(this.chunked_last_value.is_none());

                *this.chunked_last_value = chunk.values.pop().and_then(|value| value.kind);
            }

            let fields = this
                .field_index
                .as_ref()
                .expect("would have errored out by now if not set");

            // if we can construct at least 1 row, return a ResultIter, otherwise we
            // need to loop and try and get the next PartialResultSet.
            if let Some(iter) = build_result_iter_from_partial(
                this.existing_iter,
                fields,
                this.partial_row,
                chunk.values,
                chunk.stats,
            ) {
                return Poll::Ready(Some(Ok(iter)));
            }
        }
    }
}

fn build_result_iter_from_partial<T: Table>(
    existing_iter: &mut Option<ResultIter<T>>,
    fields: &FieldIndex<T::NumColumns>,
    partial_row: &mut Vec<protobuf::Value>,
    mut new_chunk: Vec<protobuf::Value>,
    stats: Option<spanner::ResultSetStats>,
) -> Option<ResultIter<T>> {
    let n_fields = fields.len();

    let needed_to_fill_row = n_fields - partial_row.len();

    // make sure we have a full rows worth of capacity if we need to insert anything
    if !partial_row.is_empty() || !new_chunk.is_empty() {
        partial_row.reserve(needed_to_fill_row);
    }

    // if this chunk can't fill up the row, append to the partial row and bail.
    if new_chunk.len() < needed_to_fill_row {
        partial_row.append(&mut new_chunk);

        // use the existing iter if it exists, and is not empty.
        return match existing_iter {
            Some(existing) if !existing.is_empty() => existing_iter.take(),
            _ => None,
        };
    }

    // otherwise, fill the partial row, then assemble a ResultIter
    let mut value_iter = new_chunk.into_iter();
    partial_row.extend(value_iter.by_ref().take(needed_to_fill_row));

    // at this point, we can use the remaining len of the value_iter to determine how many
    // full rows we'll get from this chunk (+1 from the filled 'partial_row')
    let full_rows = value_iter.len().div_floor(n_fields);

    // see if we can reuse an existing iter's fields, though we'll need to append it's rows
    // to a new vec since we can't get rows vec::IntoIter back into a vec.
    let (fields_index, mut rows) = match existing_iter.take() {
        Some(existing) => {
            let (field_index, existing_rows) = existing.into_parts();
            let mut rows = Vec::with_capacity(1 + full_rows + existing_rows.len());
            rows.extend(existing_rows);
            (field_index, rows)
        }
        None => (fields.clone(), Vec::with_capacity(1 + full_rows)),
    };

    let partial_remainder = value_iter.len() % n_fields;

    let partial_replacement = if partial_remainder == 0 {
        Vec::new()
    } else {
        Vec::with_capacity(fields.len())
    };

    // push the now-filled partial row first
    rows.push(ListValue {
        values: std::mem::replace(partial_row, partial_replacement),
    });

    // fill in as many full rows as we can
    while value_iter.len() >= n_fields {
        let mut values = Vec::with_capacity(n_fields);
        values.extend(value_iter.by_ref().take(n_fields));
        rows.push(ListValue { values });
    }

    // move the remaining partial row into the container for the next chunk
    partial_row.extend(value_iter);

    Some(ResultIter::from_parts(fields_index.clone(), rows, stats))
}

mod chunked {
    use protos::protobuf::value::Kind;
    use protos::protobuf::{ListValue, Struct, Value};

    pub fn merge_value_kind(parent: &mut Kind, new: Kind) -> Result<(), Kind> {
        use Kind::*;

        match (parent, new) {
            // numbers, booleans and nulls cant be chunked
            (NullValue(_) | NumberValue(_) | BoolValue(_), _)
            | (_, NullValue(_) | NumberValue(_) | BoolValue(_)) => Ok(()),
            // chunked strings are just concatenated
            (StringValue(parent), StringValue(new)) => {
                parent.push_str(new.as_str());
                Ok(())
            }
            // lists are concatenated w/ the last + first elements potentially merged
            (ListValue(parent), ListValue(chunked)) => merge_list(parent, chunked),
            // structs are merged and maybe recursively merged
            (StructValue(parent), StructValue(chunked)) => merge_struct(parent, chunked),
            // mismatched kinds are invalid
            (_, other) => Err(other),
        }
    }

    fn merge_list(parent: &mut ListValue, mut chunked: ListValue) -> Result<(), Kind> {
        use Kind::*;

        let parent_last = parent
            .values
            .last_mut()
            .and_then(|value| value.kind.as_mut());

        macro_rules! append_similar {
            ($chunked:expr; $variant:ident($chunk:ident) => $blk:block) => {{
                let first = $chunked
                    .values
                    .get_mut(0)
                    .and_then(|value| value.kind.take());

                match first {
                    Some($variant($chunk)) => $blk,
                    Some(other) => {
                        $chunked.values[0].kind = Some(other);
                        return Err(Kind::ListValue($chunked));
                    }
                    None => return Err(Kind::ListValue($chunked)),
                }

                parent.values.extend($chunked.values.into_iter().skip(1));
                Ok(())
            }};
        }

        match parent_last {
            None | Some(NullValue(_) | NumberValue(_) | BoolValue(_)) => {
                parent.values.append(&mut chunked.values);
                Ok(())
            }
            Some(StringValue(parent_string)) => {
                append_similar!(chunked; StringValue(chunk) => {
                    parent_string.push_str(chunk.as_str());
                })
            }
            Some(ListValue(parent_list)) => {
                append_similar!(chunked; ListValue(subchunk) => {
                    merge_list(parent_list, subchunk)?;
                })
            }
            Some(StructValue(parent_struct)) => {
                append_similar!(chunked; StructValue(chunk_struct) => {
                    merge_struct(parent_struct, chunk_struct)?;
                })
            }
        }
    }

    fn merge_struct(parent: &mut Struct, chunked: Struct) -> Result<(), Kind> {
        for (name, value) in chunked.fields {
            match parent.fields.get_mut(&name) {
                None => {
                    parent.fields.insert(name, value);
                }
                Some(existing) => match (existing.kind.as_mut(), value.kind) {
                    (Some(existing), Some(chunk)) => {
                        merge_value_kind(existing, chunk)?;
                    }
                    (None, Some(value)) => {
                        parent.fields.insert(name, Value { kind: Some(value) });
                    }
                    (_, None) => (),
                },
            }
        }

        Ok(())
    }
}
