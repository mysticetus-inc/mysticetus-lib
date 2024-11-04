use std::fmt;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, ready};
use std::time::Instant;

use apache_avro::Schema;
use futures::{Stream, StreamExt};
use protos::bigquery_storage::ReadRowsResponse;
use protos::bigquery_storage::read_rows_response::Rows;
use serde::de::DeserializeSeed;

use super::{Error, avro_de};

#[derive(Debug)]
#[pin_project]
pub struct ReadStream<S> {
    stream_id: usize,
    #[pin]
    stream: tonic::Streaming<ReadRowsResponse>,
    schema: Arc<Schema>,
    seed: S,
    rows_read: usize,
    bytes_read: usize,
    last_yielded: Option<std::time::Instant>,
}

impl<S> ReadStream<S>
where
    for<'de> S: DeserializeSeed<'de> + Clone,
{
    pub(crate) fn new_with_seed(
        stream_id: usize,
        stream: tonic::Streaming<ReadRowsResponse>,
        schema: Arc<Schema>,
        seed: S,
    ) -> Self {
        Self {
            stream_id,
            stream,
            schema,
            seed,
            rows_read: 0,
            bytes_read: 0,
            last_yielded: None,
        }
    }

    pub async fn next_row_iter(&mut self) -> Result<Option<avro_de::RowIter<S>>, Error> {
        let resp = match self.stream.message().await? {
            Some(resp) => resp,
            None => return Ok(None),
        };

        let row_count = resp.row_count as usize;

        let rows = match resp.rows {
            Some(Rows::AvroRows(avro_rows)) if row_count > 0 => avro_rows,
            Some(_) => return Err(Error::ArrowNotSupported),
            _ => return Ok(None),
        };

        self.rows_read += row_count;
        self.bytes_read += rows.serialized_binary_rows.len();

        let reader = Cursor::new(rows.serialized_binary_rows);

        avro_de::RowsDeserializer::new(reader, self.schema.clone(), row_count)
            .map(|de| Some(de.row_iter_with_seed(self.seed.clone())))
    }

    pub async fn next_batch<O>(&mut self) -> Result<Option<Vec<O>>, Error>
    where
        for<'de> S: DeserializeSeed<'de, Value = O>,
    {
        self.next().await.transpose()
    }
}

impl<S, O> Stream for ReadStream<S>
where
    for<'de> S: DeserializeSeed<'de, Value = O> + Clone,
{
    type Item = Result<Vec<O>, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        let resp = match ready!(this.stream.poll_next(cx)) {
            Some(Ok(resp)) => resp,
            Some(Err(err)) => return Poll::Ready(Some(Err(Error::from(err)))),
            None => return Poll::Ready(None),
        };

        let row_count = resp.row_count as usize;

        let rows = match resp.rows {
            Some(Rows::AvroRows(avro_rows)) if row_count > 0 => avro_rows,
            Some(_) => return Poll::Ready(Some(Err(Error::ArrowNotSupported))),
            _ => return Poll::Pending,
        };

        *this.rows_read += row_count;
        *this.bytes_read += rows.serialized_binary_rows.len();

        if let Some(last_yielded) = this.last_yielded.replace(Instant::now()) {
            let dur = last_yielded.elapsed();

            debug!(
                stream_id = this.stream_id,
                last_yielded_nanos = dur.as_nanos() as usize,
                bytes_read = tracing::field::display(HumanReadableBytes(*this.bytes_read)),
                rows_read = this.rows_read,
            );
        }
        let reader = Cursor::new(rows.serialized_binary_rows);

        let rows = avro_de::RowsDeserializer::new(reader, this.schema.clone(), row_count)
            .and_then(|de| de.consume_with_seed(this.seed));

        Poll::Ready(Some(rows))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct HumanReadableBytes(pub usize);

impl fmt::Display for HumanReadableBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut bytes = self.0 as f64;
        let mut count = 0;

        while bytes >= 1024.0 {
            bytes /= 1024.0;
            count += 1;
        }

        match count {
            0 => write!(formatter, "{bytes:.2} bytes"),
            1 => write!(formatter, "{bytes:.2} KiB"),
            2 => write!(formatter, "{bytes:.2} MiB"),
            3 => write!(formatter, "{bytes:.2} GiB"),
            4 => write!(formatter, "{bytes:.2} TiB"),
            5 => write!(formatter, "{bytes:.2} PiB"),
            _ => write!(formatter, "{bytes:.2} * 1024^{count} bytes"),
        }
    }
}
