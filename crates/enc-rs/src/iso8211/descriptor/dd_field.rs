use std::io::Read;
use std::sync::Arc;

use data_structures::small_str::SmallStr;
use data_structures::tree::Tree;
use intern::local::LocalInterner;
use intern::{InternedStr, Interner};

use super::ParseFieldDescriptor;
use super::field_controls::{Cardinality, FieldControls};
use super::format_controls::FormatControl;
use crate::Iso8211Reader;
use crate::iso8211::directory::DirectoryEntry;
use crate::iso8211::error::Iso8211ErrorKind;
use crate::iso8211::leader::{DataDescriptiveLeader, Leader};
use crate::iso8211::record::Value;
use crate::iso8211::{Iso8211Error, terminator};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DataDescriptiveField<K = Field> {
    tag: InternedStr,
    field_controls: FieldControls,
    kind: K,
}

pub(crate) trait AsFieldKind {
    fn as_field_kind(&self) -> &FieldKind;
}

impl<T: AsFieldKind> AsFieldKind for &T {
    fn as_field_kind(&self) -> &FieldKind {
        T::as_field_kind(self)
    }
}

impl<K> DataDescriptiveField<K> {
    pub(super) fn new(tag: InternedStr, field_controls: FieldControls, kind: K) -> Self {
        Self {
            tag,
            field_controls,
            kind,
        }
    }

    pub fn tag(&self) -> &InternedStr {
        &self.tag
    }

    pub fn kind(&self) -> &K {
        &self.kind
    }

    pub fn field_controls(&self) -> &FieldControls {
        &self.field_controls
    }

    pub(crate) fn read_value<R: Read>(
        &self,
        reader: &mut Iso8211Reader<R>,
        interner: &mut LocalInterner,
        dir_entry: &DirectoryEntry,
    ) -> Result<Value, Iso8211Error>
    where
        K: AsFieldKind,
    {
        match self.kind.as_field_kind() {
            // Scalar value
            FieldKind::Elementary(subfield) => {
                Value::read_from(reader, interner, &subfield.format_control)
            }
            // Single row of values
            FieldKind::Vector { subfields } => {
                let row = read_repeated_subfields(reader, interner, subfields)?;
                Ok(Value::Array(row))
            }
            // Possibly many rows of values
            FieldKind::Array { subfields } => {
                let mut rows = Vec::new();
                let start_pos = reader.position();
                loop {
                    let read = reader.position() - start_pos;

                    if read >= dir_entry.length {
                        break;
                    } else if dir_entry.length - read == 1 {
                        let b = reader.read_byte()?;
                        assert!(b == super::terminator::UNIT || b == super::terminator::FIELD);

                        break;
                    }

                    let row = read_repeated_subfields(reader, interner, subfields)?;

                    rows.push(Value::Array(row));
                }

                Ok(Value::Array(rows))
            }
            // No clue
            FieldKind::Concatenated => todo!("FieldKind::Concatenated"),
        }
    }
}

fn read_repeated_subfields<R: Read>(
    reader: &mut Iso8211Reader<R>,
    interner: &mut LocalInterner,
    subfields: &[Subfield<InternedStr>],
) -> Result<Vec<Value>, Iso8211Error> {
    let mut row = Vec::with_capacity(subfields.len());

    for subfield in subfields {
        let value = Value::read_from(reader, interner, &subfield.format_control)?;
        row.push(value);
    }

    Ok(row)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FileControl {
    file_title: Option<String>,
    field_tags: Tree<SmallStr<4>>,
}

impl ParseFieldDescriptor for FileControl {
    fn parse<R: Read>(
        reader: &mut Iso8211Reader<R>,
        _entry: &DirectoryEntry,
        leader: &DataDescriptiveLeader,
        _field_controls: &FieldControls,
        _interned: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error> {
        let file_title = if let Some(terminator::UNIT) = reader.peek_byte()? {
            // advance past the unit terminator
            reader.read_byte()?;
            None
        } else {
            reader
                .read_str_until(terminator::UNIT)
                .map(|s| Some(s.to_owned()))?
        };

        let mut field_tag_pairs: Vec<(SmallStr<4>, SmallStr<4>)> = Vec::new();

        let tag_size = leader.entry_map().size_of_tag_field() as usize;
        while reader.peek_byte()? != Some(terminator::FIELD) {
            let a = reader.read_sized_str(tag_size)?.into();
            let b = reader.read_sized_str(tag_size)?.into();

            field_tag_pairs.push((a, b));
        }
        let field_tags = Tree::from_preorder_pairs(field_tag_pairs)
            .map_err(|_| Iso8211ErrorKind::InvalidPreOrderPairs)?;

        Ok(Self {
            field_tags,
            file_title,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TmpReadStr {
    s: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Field {
    field_name: Option<InternedStr>,
    kind: FieldKind,
}

impl AsFieldKind for Field {
    fn as_field_kind(&self) -> &FieldKind {
        &self.kind
    }
}

impl Field {
    pub fn kind(&self) -> &FieldKind {
        &self.kind
    }

    pub fn field_name(&self) -> Option<&str> {
        self.field_name.as_deref()
    }

    pub fn min_length(&self) -> Option<usize> {
        match &self.kind {
            FieldKind::Elementary(sub) => sub.format_control.min_length(),
            FieldKind::Array { subfields } | FieldKind::Vector { subfields } => {
                let mut sum = 0;
                for sub in subfields {
                    if let Some(min) = sub.format_control.min_length() {
                        sum += min;
                    }
                }
                Some(sum)
            }
            FieldKind::Concatenated => None,
        }
    }

    pub fn known_length(&self) -> Option<usize> {
        match &self.kind {
            FieldKind::Elementary(sub) => sub.format_control.known_length(),
            FieldKind::Vector { subfields } => {
                let mut sum = 0;
                for sub in subfields {
                    sum += sub.format_control.known_length()?;
                }
                Some(sum)
            }
            FieldKind::Array { .. } | FieldKind::Concatenated => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Subfield<L> {
    pub label: L,
    pub format_control: FormatControl,
}

// Elementary subfield
impl Subfield<Option<Arc<str>>> {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum FieldKind {
    Elementary(Subfield<Option<InternedStr>>),
    Vector {
        subfields: Vec<Subfield<InternedStr>>,
    },
    Array {
        subfields: Vec<Subfield<InternedStr>>,
    },
    Concatenated,
}

impl ParseFieldDescriptor for Field {
    fn parse<R: Read>(
        reader: &mut Iso8211Reader<R>,
        _entry: &DirectoryEntry,
        _leader: &DataDescriptiveLeader,
        field_controls: &FieldControls,
        interner: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error> {
        let name = reader.read_str_until(terminator::UNIT)?;
        let field_name = if name.is_empty() {
            None
        } else {
            Some(interner.intern_str(name))
        };

        let kind = match field_controls.cardinality {
            Cardinality::Zero => parse_scalar_field_kind(field_controls, reader, interner)?,
            Cardinality::One => {
                let subfields = parse_repeated_subfields(field_controls, reader, interner)?;
                FieldKind::Vector { subfields }
            }
            Cardinality::TwoOrMore => {
                let subfields = parse_repeated_subfields(field_controls, reader, interner)?;
                FieldKind::Array { subfields }
            }
            Cardinality::Concatenated => todo!("concatenated"),
        };

        Ok(Self { field_name, kind })
    }
}

fn parse_scalar_field_kind<R: Read>(
    field_controls: &FieldControls,
    reader: &mut Iso8211Reader<R>,
    interner: &mut LocalInterner,
) -> Result<FieldKind, Iso8211Error> {
    let raw = reader.read_str_until(terminator::UNIT)?;
    let label = if raw.is_empty() {
        None
    } else {
        Some(interner.intern_str(raw))
    };

    let mut fmt_ctrls = FormatControl::from_reader(reader, field_controls, interner, Ok)?;
    assert_eq!(fmt_ctrls.len(), 1);

    Ok(FieldKind::Elementary(Subfield {
        label,
        format_control: fmt_ctrls.remove(0),
    }))
}

fn parse_repeated_subfields<R: Read>(
    field_controls: &FieldControls,
    reader: &mut Iso8211Reader<R>,
    interner: &mut LocalInterner,
) -> Result<Vec<Subfield<InternedStr>>, Iso8211Error> {
    // use rsplit instead, that way we can pop from the end and get things in order.
    // we need to do this because we cant hold onto this string, and continue parsing
    // format controls at the same time.
    let mut labels = reader
        .read_str_until(terminator::UNIT)?
        .rsplit('!')
        .map(|s| interner.intern_str(s))
        .collect::<Vec<_>>();

    FormatControl::from_reader(reader, field_controls, interner, |format_control| {
        let label = labels
            .pop()
            .ok_or_else(|| Iso8211Error::eof("mismatched array descriptor label count"))?;

        Ok(Subfield {
            label,
            format_control,
        })
    })
}

impl ParseFieldDescriptor for TmpReadStr {
    fn parse<R: Read>(
        reader: &mut Iso8211Reader<R>,
        entry: &DirectoryEntry,
        _leader: &DataDescriptiveLeader,
        _field_controls: &FieldControls,
        _interned: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error> {
        let s = reader
            .read_sized_str(entry.length as usize - 9 - 1)?
            .to_owned();
        Ok(Self { s })
    }
}

macro_rules! impl_tmp_ident {
    ($($field:ident),* $(,)?) => {
        $(

            #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
            pub struct $field {
                tmp: Field,
            }

            impl ParseFieldDescriptor for $field {
                fn parse<R: Read>(
                    reader: &mut Iso8211Reader<R>,
                    entry: &DirectoryEntry,
                    leader: &DataDescriptiveLeader,
                    field_controls: &FieldControls,
                    interner: &mut LocalInterner,
                ) -> Result<Self, Iso8211Error> {
                    let tmp = Field::parse(reader, entry, leader, field_controls, interner)?;
                    Ok(Self { tmp })
                }
            }

            impl AsFieldKind for $field {
                fn as_field_kind(&self) -> &FieldKind {
                    self.tmp.as_field_kind()
                }
            }
        )*
    };
}

impl_tmp_ident! {
    RecordIdentifier,
    UserApplication,
    RecursiveLinks,
    AnnouncerSequence,
}
