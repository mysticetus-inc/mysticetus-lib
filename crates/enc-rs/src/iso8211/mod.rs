mod descriptor;
mod directory;
pub mod error;
mod leader;

use std::io::{BufRead, Read};

pub use descriptor::dd_record::DataDescriptiveRecord;
pub use error::Iso8211Error;
use intern::local::LocalInterner;

pub mod record;
use self::leader::Leader;
use self::record::{FieldIter, LogicalRecord};
use crate::reader::Iso8211Reader;

pub(crate) mod terminator {
    pub const FIELD: u8 = 0x1E;
    pub const UNIT: u8 = 0x1F;

    const FIELD_PRINT_SYMBOL: u8 = b';';
    const UNIT_PRINT_SYMBOL: u8 = b'&';
}

pub(crate) mod delimiter {
    pub const SUBFIELD_LABEL: u8 = b'!';
    pub const VECTOR_LABEL: u8 = b'*';
    pub const ARRAY_DESC: u8 = b'\\';
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Iso8211File {
    ddr: DataDescriptiveRecord,
    records: Vec<LogicalRecord>,
}

impl Iso8211File {
    pub fn read<R>(
        mut reader: Iso8211Reader<R>,
        interner: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error>
    where
        R: Read,
    {
        let ddr = DataDescriptiveRecord::from_reader(&mut reader, interner)?;

        println!("{ddr:#?}");

        assert_eq!(ddr.leader().record_length(), reader.position());

        let mut records = Vec::new();

        let mut last_pos = 0;

        loop {
            let curr_pos = reader.position();
            if !reader.has_data_left()? || curr_pos == last_pos {
                break;
            }
            println!("reading logical record from {curr_pos} bytes");
            last_pos = curr_pos;
            let rec = record::LogicalRecord::from_reader(&mut reader, &ddr, interner)?;
            records.push(rec);
        }

        Ok(Self { ddr, records })
    }

    fn iter(&self) -> RecordIter<'_> {
        RecordIter {
            records: self.records.iter(),
            ddr: &self.ddr,
        }
    }
}

pub struct RecordIter<'a> {
    records: std::slice::Iter<'a, LogicalRecord>,
    ddr: &'a DataDescriptiveRecord,
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = FieldIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let rec = self.records.next()?;
        Some(rec.iter(self.ddr))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    const TEST_ROOT_FILE: &str = "./tests/ENC_ROOT/US5OR01M/US5OR01M.000";

    struct SerializeIter<T>(RefCell<T>);

    impl<T> serde::Serialize for SerializeIter<T>
    where
        T: Iterator,
        T::Item: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeSeq;

            let mut seq = serializer.serialize_seq(None)?;

            for e in &mut *self.0.borrow_mut() {
                seq.serialize_element(&e)?;
            }

            seq.end()
        }
    }

    #[test]
    fn test_parse() -> Result<(), Iso8211Error> {
        let mut interner = LocalInterner::new();
        let file = Iso8211Reader::new(std::fs::File::open(TEST_ROOT_FILE)?);
        let parsed = Iso8211File::read(file, &mut interner)?;

        let ser = SerializeIter(RefCell::new(parsed.iter().flat_map(|i| i)));

        let mut dst = std::fs::File::create("test.json")?;
        serde_json::to_writer_pretty(&mut dst, &ser).unwrap();

        Ok(())
    }
}
