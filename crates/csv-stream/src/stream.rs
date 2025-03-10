use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use bytes::{Buf, BufMut, BytesMut};
use futures::{Stream, TryStream};

use crate::reader::{RawRow, ReadResult};
use crate::{Error, Row};

pin_project_lite::pin_project! {
    pub struct CsvStream<S: TryStream> {
        state: StreamState,
        current_buf: Option<S::Ok>,
        #[pin]
        stream: Option<S>,
    }
}

struct StreamState {
    reader: crate::reader::CsvReader,
    contig: Option<BytesMut>,
    headers: Option<Arc<RawRow>>,
}

impl StreamState {
    fn read_from<B: Buf + ?Sized>(&mut self, buf: &mut B) -> Option<Row> {
        loop {
            let row = match self.contig.as_mut() {
                Some(contig) if contig.has_remaining() => match self.reader.read_some(contig) {
                    ReadResult::ReadRow(row) => row,
                    ReadResult::InputNotContiguous | ReadResult::NeedsMoreData => {
                        if !buf.has_remaining() {
                            break None;
                        }
                        contig.put(&mut *buf);
                        continue;
                    }
                },
                _ => match self.reader.read_some(buf) {
                    ReadResult::ReadRow(row) => row,
                    ReadResult::NeedsMoreData => return None,
                    ReadResult::InputNotContiguous => {
                        let contig_dst = self
                            .contig
                            .get_or_insert_with(|| BytesMut::with_capacity(buf.remaining()));
                        contig_dst.put(&mut *buf);
                        continue;
                    }
                },
            };

            match self.headers.as_ref() {
                None => self.headers = Some(Arc::new(row)),
                Some(headers) => return Some(Row::new(Arc::clone(headers), row)),
            }
        }
    }
}

impl<S> CsvStream<S>
where
    S: TryStream,
    S::Ok: Buf,
{
    pub fn new(stream: S) -> Self {
        Self {
            state: StreamState {
                reader: crate::reader::CsvReader::new(),
                contig: None,
                headers: None,
            },
            current_buf: None,
            stream: Some(stream),
        }
    }
}

impl<S> Stream for CsvStream<S>
where
    S: Stream<Item = Result<S::Ok, S::Error>> + TryStream,
    S::Ok: Buf,
{
    type Item = Result<Row, Error<S::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match (this.current_buf.as_mut(), this.stream.as_mut().as_pin_mut()) {
                (None, Some(stream)) => match std::task::ready!(stream.poll_next(cx)) {
                    Some(result) => *this.current_buf = Some(result.map_err(Error::Stream)?),
                    None => this.stream.set(None),
                },
                (Some(buf), _) => {
                    let read_result = this.state.read_from(buf);
                    if !buf.has_remaining() {
                        *this.current_buf = None;
                    }

                    if let Some(row) = read_result {
                        break Poll::Ready(Some(Ok(row)));
                    }
                }
                (None, None) => break Poll::Ready(None),
            }
        }
    }
}
