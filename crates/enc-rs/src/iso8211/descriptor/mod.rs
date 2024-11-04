pub(crate) mod array_descriptor;
pub(crate) mod dd_field;
pub(crate) mod dd_record;
pub(crate) mod field_controls;
pub(crate) mod format_controls;

use std::io::Read;

pub use dd_field::DataDescriptiveField;
use field_controls::FieldControls;
use intern::local::LocalInterner;

use super::directory::DirectoryEntry;
use super::error::Iso8211Error;
use super::leader::DataDescriptiveLeader;
use super::terminator;
use crate::Iso8211Reader;

trait ParseFieldDescriptor: Sized {
    fn parse<R: Read>(
        reader: &mut Iso8211Reader<R>,
        entry: &DirectoryEntry,
        leader: &DataDescriptiveLeader,
        field_controls: &FieldControls,
        interned: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error>;

    fn parse_ddf<R: Read>(
        reader: &mut Iso8211Reader<R>,
        entry: &DirectoryEntry,
        leader: &DataDescriptiveLeader,
        interned: &mut LocalInterner,
    ) -> Result<DataDescriptiveField<Self>, Iso8211Error> {
        let init = reader.position();

        assert!(leader.field_control_length() == 9);
        let field_controls = reader.parse_from_array::<FieldControls, 9>()?;

        let kind = Self::parse(reader, entry, leader, &field_controls, interned)?;

        // read past the field terminator.

        match entry.length.abs_diff(reader.position() - init) {
            0 => (),
            1 => assert_eq!(reader.read_byte()?, terminator::FIELD),
            n => panic!("read {n} bytes too many/few"),
        }

        // make sure we read the expected number of bytes.
        let bytes_read = reader.position() - init;
        assert_eq!(entry.length, bytes_read);

        Ok(DataDescriptiveField::new(
            entry.tag.clone(),
            field_controls,
            kind,
        ))
    }
}
