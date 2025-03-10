use std::fmt;
use std::num::NonZeroUsize;

use bytes::{Buf, Bytes};
use csv_core::{ReadRecordResult, Reader};

use crate::buffer::Buffer;

const DEFAULT_INDICES_CAPACITY: usize = std::mem::size_of::<usize>() * 32;
const DEFAULT_BUF_CAPACITY: usize = 4096;

pub struct CsvReader {
    reader: Box<Reader>,
    record_state: ReadRecordResult,
    data: Buffer,
    // We reuse the same buffer type as the data itself,
    // but this should be treated as a buffer of usize's, not bytes
    indices: Buffer,
}

impl CsvReader {
    pub fn new() -> Self {
        Self {
            reader: Box::new(Reader::new()),
            record_state: ReadRecordResult::InputEmpty,
            data: Buffer::with_capacity(DEFAULT_BUF_CAPACITY),
            indices: Buffer::with_capacity(DEFAULT_INDICES_CAPACITY),
        }
    }

    fn pop_row(&mut self) -> RawRow {
        RawRow {
            line: NonZeroUsize::new(self.reader.line() as usize - 1).unwrap_or(NonZeroUsize::MIN),
            data: self.data.take(),
            indices: self.indices.take(),
        }
    }

    pub fn read_some<B: Buf + ?Sized>(&mut self, buf: &mut B) -> ReadResult {
        let mut last_remaining = buf.remaining();

        while buf.has_remaining() {
            let indices_dst = bytemuck::cast_slice_mut(self.indices.chunk_mut());

            let (record_res, input_read, output_written, idx_written) =
                self.reader
                    .read_record(buf.chunk(), self.data.chunk_mut(), indices_dst);

            buf.advance(input_read);
            self.data.inc_len(output_written);
            self.indices
                .inc_len(std::mem::size_of::<usize>() * idx_written);
            self.record_state = record_res;

            match self.record_state {
                ReadRecordResult::Record => return ReadResult::ReadRow(self.pop_row()),
                ReadRecordResult::OutputFull => self.data.grow(),
                ReadRecordResult::OutputEndsFull => self.indices.grow(),
                // the input buf might not actually be empty, this variant is also
                // returned when the chunk given by buf doesnt contain the end of record
                // (i.e we need to read more).
                ReadRecordResult::InputEmpty => {
                    // if we didn't read anything for 2 iterations in a row,
                    // we know the input buffer isnt contiguous, and despite there
                    // being remaining data, we can't get to it since the end of record
                    // terminator cant be found.
                    if last_remaining == buf.remaining() {
                        // sanity assert that only a non-contiguous buffer
                        // could make happen
                        assert!(buf.chunk().len() < buf.remaining());
                        return ReadResult::InputNotContiguous;
                    }
                }
                ReadRecordResult::End => {
                    unreachable!("the input should never be empty (has_remaining is true)")
                }
            }

            last_remaining = buf.remaining();
        }

        ReadResult::NeedsMoreData
    }
}

#[derive(Debug)]
pub enum ReadResult {
    ReadRow(RawRow),
    NeedsMoreData,
    InputNotContiguous,
}

#[derive(Clone)]
pub struct RawRow {
    line: NonZeroUsize,
    indices: Bytes,
    data: Bytes,
}

impl RawRow {
    pub fn line(&self) -> NonZeroUsize {
        self.line
    }
    fn indices(&self) -> &[usize] {
        bytemuck::cast_slice(&self.indices)
    }

    fn get(&self, column: usize) -> Option<&[u8]> {
        let start = self.indices().get(column).copied()?;
        let end = self.indices().get(column + 1).copied()?;
        Some(&self.data[start..end])
    }
}

impl fmt::Debug for RawRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawRow")
            .field("line", &self.line)
            .field("values", &self.into_iter())
            .finish()
    }
}

impl<'a> IntoIterator for &'a RawRow {
    type Item = &'a bstr::BStr;
    type IntoIter = RawRowIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RawRowIter {
            row: self,
            index: 0,
        }
    }
}

pub struct RawRowIter<'a> {
    row: &'a RawRow,
    index: usize,
}

impl fmt::Debug for RawRowIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from_beginning = Self {
            row: self.row,
            index: 0,
        };

        f.debug_list().entries(from_beginning).finish()
    }
}

impl<'a> Iterator for RawRowIter<'a> {
    type Item = &'a bstr::BStr;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.row.get(index).map(bstr::BStr::new)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let slots = self.row.indices.len() / 4;
        let rem = slots.saturating_sub(self.index);
        (rem, Some(rem))
    }
}

impl ExactSizeIterator for RawRowIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::HorribleBuf;

    #[test]
    fn test_raw_reader() -> std::io::Result<()> {
        const FILE: &str = "/home/mrudisel/src/mysticloud/media/IOS21ALGP82Q/\
                            DetectionSummary-Island_Pride-2000-01-01-2024-05-09.csv";

        let mut input = HorribleBuf::new(std::fs::read(FILE)?);

        let mut reader = CsvReader::new();

        loop {
            match reader.read_some(&mut input) {
                ReadResult::ReadRow(row) => println!("{row:#?}"),
                ReadResult::NeedsMoreData if !input.has_remaining() => break Ok(()),
                ReadResult::NeedsMoreData => (),
                ReadResult::InputNotContiguous => todo!("bad buffer Lol"),
            }
        }
    }
}
