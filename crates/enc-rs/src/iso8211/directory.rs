use std::io::Read;

use intern::local::LocalInterner;
use intern::{InternedStr, Interner};

use super::leader::Leader;
use super::{Iso8211Error, terminator};
use crate::Iso8211Reader;
use crate::utils::chars_to_usize;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DirectoryEntries {
    pub(crate) entries: Vec<DirectoryEntry>,
}

impl DirectoryEntries {
    pub(crate) fn from_reader<R, L>(
        reader: &mut Iso8211Reader<R>,
        leader: &L,
        interner: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error>
    where
        R: Read,
        L: Leader,
    {
        let entry_map = leader.entry_map();
        let entry_count = leader.dictionary_entry_len() / entry_map.total_size();

        let mut entries = Vec::with_capacity(entry_count);

        let tag_len = entry_map.size_of_tag_field() as usize;
        let field_length_len = entry_map.size_of_length_field() as usize;
        let pos_len = entry_map.size_of_pos_field() as usize;

        for _ in 0..entry_count {
            let tag = reader.read_sized_str(tag_len)?;
            let tag = interner.intern_str(tag);
            let length = reader.read_bytes(field_length_len).map(chars_to_usize)??;
            let pos = reader.read_bytes(pos_len).map(chars_to_usize)??;

            entries.push(DirectoryEntry {
                length: length as u64,
                position: pos as u64,
                tag,
            });
        }

        // read past the field terminator
        assert_eq!(reader.read_byte()?, terminator::FIELD);

        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DirectoryEntry {
    pub(crate) tag: InternedStr,
    pub(crate) length: u64,
    pub(crate) position: u64,
}

impl PartialOrd for DirectoryEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.position.partial_cmp(&other.position)
    }
}

impl Ord for DirectoryEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}
