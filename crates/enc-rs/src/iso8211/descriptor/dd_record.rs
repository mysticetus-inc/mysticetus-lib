use std::collections::BTreeMap;
use std::io::Read;
use std::sync::Arc;

use intern::InternedStr;
use intern::local::LocalInterner;

use crate::Iso8211Reader;
use crate::iso8211::descriptor::ParseFieldDescriptor;
use crate::iso8211::descriptor::dd_field::{self, DataDescriptiveField, Field};
use crate::iso8211::directory::{DirectoryEntries, DirectoryEntry};
use crate::iso8211::error::{Iso8211Error, Iso8211ErrorKind};
use crate::iso8211::leader::{DataDescriptiveLeader, Leader};
use crate::iso8211::record::Value;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DataDescriptiveRecord {
    leader: DataDescriptiveLeader,
    directory: DirectoryEntries,
    file_control_field: Arc<DataDescriptiveField<dd_field::FileControl>>,
    record_identifier: Option<Arc<DataDescriptiveField<dd_field::RecordIdentifier>>>,
    user_application: Option<Arc<DataDescriptiveField<dd_field::UserApplication>>>,
    announcer_seq: Option<Arc<DataDescriptiveField<dd_field::AnnouncerSequence>>>,
    recursive_links: Option<Arc<DataDescriptiveField<dd_field::RecursiveLinks>>>,
    data_descriptive_fields: BTreeMap<InternedStr, Arc<DataDescriptiveField>>,
}

impl DataDescriptiveRecord {
    pub fn file_control_field(&self) -> &DataDescriptiveField<dd_field::FileControl> {
        &self.file_control_field
    }

    pub fn get_ddf(&self, field: &InternedStr) -> Option<&Arc<DataDescriptiveField>> {
        self.data_descriptive_fields.get(field)
    }

    pub(crate) fn parse_ddf_value<R: Read>(
        &self,
        reader: &mut Iso8211Reader<R>,
        interner: &mut LocalInterner,
        dir_entry: &DirectoryEntry,
    ) -> Result<Value, Iso8211Error> {
        println!("TAG: {}", dir_entry.tag);
        let field_start = reader.position();

        let value = match &*dir_entry.tag {
            "0000" => todo!("fcf"),
            "0001" => self
                .record_identifier
                .as_ref()
                .expect("no record_identifier found")
                .read_value(reader, interner, dir_entry)?,
            "0002" => self
                .user_application
                .as_ref()
                .expect("no user_application found")
                .read_value(reader, interner, dir_entry)?,
            "0003" => self
                .announcer_seq
                .as_ref()
                .expect("no announcer_segment found")
                .read_value(reader, interner, dir_entry)?,
            "0004" | "0005" | "0006" | "0007" | "0008" => panic!("cannot request reserved tag"),
            "0009" => self
                .recursive_links
                .as_ref()
                .expect("no recursive_links found")
                .read_value(reader, interner, dir_entry)?,
            _ => self
                .data_descriptive_fields
                .get(&dir_entry.tag)
                .expect("no ddf found")
                .read_value(reader, interner, dir_entry)?,
        };

        let delta = reader.position() - field_start;

        if delta > dir_entry.length {
            panic!(
                "read {} bytes when we should have only read {}",
                delta, dir_entry.length
            );
        } else if dir_entry.length - delta == 1 {
            // if there's one byte remaining, it's likely that we didnt need to read
            // the terminator byte.
            let last = reader.read_byte()?;
            assert!(last == super::terminator::UNIT || last == super::terminator::FIELD);
        }

        Ok(value)
    }

    pub fn leader(&self) -> &DataDescriptiveLeader {
        &self.leader
    }

    pub(crate) fn from_reader<R>(
        reader: &mut Iso8211Reader<R>,
        interner: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error>
    where
        R: Read,
    {
        let init = reader.position();
        let leader = reader.parse_from_array::<_, 24>()?;
        let directory = DirectoryEntries::from_reader(reader, &leader, interner)?;

        let mut file_control_field = None;
        let mut record_identifier = None;
        let mut user_application = None;
        let mut announcer_seq = None;
        let mut recursive_links = None;

        let mut data_descriptive_fields = BTreeMap::new();

        for entry in directory.entries.iter() {
            println!("starting record {}", entry.tag);
            match entry.tag.as_ref() {
                "0000" => {
                    let fcf = dd_field::FileControl::parse_ddf(reader, entry, &leader, interner)?;
                    file_control_field = Some(Arc::new(fcf));
                }
                "0001" => {
                    let ri =
                        dd_field::RecordIdentifier::parse_ddf(reader, entry, &leader, interner)?;
                    record_identifier = Some(Arc::new(ri));
                }
                "0002" => {
                    let ua =
                        dd_field::UserApplication::parse_ddf(reader, entry, &leader, interner)?;
                    user_application = Some(Arc::new(ua));
                }
                "0003" => {
                    let aseq =
                        dd_field::AnnouncerSequence::parse_ddf(reader, entry, &leader, interner)?;
                    announcer_seq = Some(Arc::new(aseq));
                }
                "0004" | "0005" | "0006" | "0007" | "0008" => {
                    return Err("found reserved tag in directory".into());
                }
                "0009" => {
                    let rl = dd_field::RecursiveLinks::parse_ddf(reader, entry, &leader, interner)?;
                    recursive_links = Some(Arc::new(rl));
                }
                _ => {
                    let ddf = Field::parse_ddf(reader, entry, &leader, interner)?;
                    data_descriptive_fields.insert(ddf.tag().clone(), Arc::new(ddf));
                }
            }
        }

        let file_control_field = file_control_field
            .ok_or_else(|| Iso8211ErrorKind::Misc("no file control field in ddf".into()))?;

        let bytes_read = reader.position() - init;
        assert_eq!(bytes_read, leader.record_length());

        Ok(Self {
            leader,
            directory,
            data_descriptive_fields,
            file_control_field,
            recursive_links,
            record_identifier,
            user_application,
            announcer_seq,
        })
    }

    /*
    pub(crate) fn visit_fmt_controls<F>(&self, mut f: F)
    where
        F: FnMut(&FormatControl),
    {
        macro_rules! visit_fmt {
            ($f:expr, $t:expr) => {{
                if let Some(ref r) = $t {
                    for fmt in r.kind.tmp.format_controls.iter() {
                        $f(fmt);
                    }
                }
            }};
        }

        visit_fmt!(f, self.record_identifier);
        visit_fmt!(f, self.user_application);
        visit_fmt!(f, self.announcer_seq);
        visit_fmt!(f, self.recursive_links);

        for dr in self.data_descriptive_fields.iter() {
            for fmt in dr.kind.format_controls.iter() {
                f(fmt);
            }
        }
    }
    */
}
