use std::io::Read;
use std::sync::Arc;

use intern::local::LocalInterner;
use intern::{InternedStr, Interner};

use super::field_controls::Cardinality;
use crate::iso8211::error::Iso8211ErrorKind;
use crate::iso8211::{Iso8211Error, terminator};
use crate::utils::byte_enum;
use crate::{FromByte, Iso8211Reader};

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ArrayDescDelim {
        Vector = b'*',
        Concat = b'\\',
    }

    from_byte_error(bad_byte: u8) -> Iso8211Error {
        Iso8211ErrorKind::InvalidDelimiter { found: bad_byte, parsing: "ArrayDescDelim" }.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum FullDelim {
    Label = b'!',
    Vector = b'*',
    Concat = b'\\',
}

impl FullDelim {
    fn from_byte_opt(b: u8) -> Option<Self> {
        match b {
            b'!' => Some(Self::Label),
            b'*' => Some(Self::Vector),
            b'\\' => Some(Self::Concat),
            _ => None,
        }
    }

    fn find_next(s: &str) -> Option<(usize, Self)> {
        s.as_bytes().iter().enumerate().find_map(|(idx, b)| {
            if s.is_char_boundary(idx) {
                match Self::from_byte_opt(*b) {
                    Some(delim) => Some((idx, delim)),
                    None => None,
                }
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayDescriptors {
    dims: Vec<Vec<InternedStr>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayDescriptor {
    Numeric(),
    Label(ArrayDescriptorLabel),
    Concatenated(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayDescriptorLabel {
    Subfield(Option<InternedStr>),
    Vector(Vec<InternedStr>),
    Cartesian(Vec<InternedStr>),
}

impl ArrayDescriptor {
    pub(crate) fn from_reader<R: Read>(
        reader: &mut Iso8211Reader<R>,
        field_controls: &super::FieldControls,
        interner: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error> {
        let s = reader.read_str_until(terminator::UNIT)?;

        match field_controls.cardinality {
            Cardinality::Zero if s.is_empty() => {
                Ok(ArrayDescriptor::Label(ArrayDescriptorLabel::Subfield(None)))
            }
            Cardinality::Zero => Ok(ArrayDescriptor::Label(ArrayDescriptorLabel::Subfield(
                Some(interner.intern_str(s)),
            ))),
            Cardinality::One => {
                assert_ne!(s.chars().next(), Some('*'));
                let vec_labels = s
                    .split('!')
                    .map(|vec_label| interner.intern_str(vec_label))
                    .collect::<Vec<_>>();
                Ok(ArrayDescriptor::Label(ArrayDescriptorLabel::Vector(
                    vec_labels,
                )))
            }
            Cardinality::TwoOrMore => {
                assert_eq!(s.chars().next(), Some('*'));
                let cartesian_labels = s[1..]
                    .split('!')
                    .map(|cart_label| interner.intern_str(cart_label))
                    .collect::<Vec<_>>();
                Ok(ArrayDescriptor::Label(ArrayDescriptorLabel::Cartesian(
                    cartesian_labels,
                )))
            }
            Cardinality::Concatenated => todo!("concatenated array descriptors: '{}'", s),
        }
    }
}

impl ArrayDescriptors {
    pub(crate) fn from_reader<R: Read>(
        reader: &mut Iso8211Reader<R>,
        _field_controls: &super::FieldControls,
        interner: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error> {
        let mut s = reader.read_str_until(terminator::UNIT)?;

        /*
        if s.is_empty() {
            return Ok(Self::Empty);
        }
        */

        let mut dims = vec![vec![]];

        let mut last_delim = FullDelim::Label;

        while let Some((delim_idx, next_delim)) = FullDelim::find_next(s) {
            let dim_vec = match last_delim {
                FullDelim::Label | FullDelim::Concat => dims.last_mut().unwrap(),
                FullDelim::Vector => {
                    dims.push(vec![]);
                    dims.last_mut().unwrap()
                }
            };

            last_delim = next_delim;

            dim_vec.push(interner.intern_str(&s[..delim_idx]));

            match s.get(delim_idx + 1..) {
                Some(rem) => s = rem,
                None => s = "",
            }
        }

        Ok(Self { dims })
    }

    // not a real function, needs to be refactored and removed.
    // used to get rid of compiler errors workspace-wide
    pub fn iter(&self) -> std::slice::Iter<'_, ArrayDesc> {
        [].iter()
    }

    pub fn dims(&self) -> usize {
        self.dims.len()
    }

    pub fn total_labels(&self) -> usize {
        self.dims.iter().map(|row| row.len()).product::<usize>()
    }

    pub fn visit_elems<F>(&self, mut visit_fn: F)
    where
        F: FnMut(&[Arc<str>]),
    {
        let mut current_idxes = vec![0_usize; self.dims.len()];
        let buf = Vec::with_capacity(self.dims.len());

        loop {
            current_idxes.iter_mut().for_each(|idx| *idx = 0);

            visit_fn(&buf);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayDesc {
    modifier: Option<ArrayDescDelim>,
    name: InternedStr,
}

impl ArrayDesc {
    pub(crate) fn from_reader<R: Read>(
        reader: &mut Iso8211Reader<R>,
        interner: &mut LocalInterner,
    ) -> Result<Vec<Self>, Iso8211Error> {
        let s = reader.read_str_until(terminator::UNIT)?;

        let descriptors = s
            .split_terminator('!')
            .filter_map(|s| Self::from_str(s, interner).transpose())
            .collect::<Result<Vec<Self>, Iso8211Error>>()?;

        Ok(descriptors)
    }

    fn from_str(s: &str, interned: &mut LocalInterner) -> Result<Option<Self>, Iso8211Error> {
        match s.len() {
            0 => return Ok(None),
            l if l < 2 => panic!("strange array descriptor string: '{}'", s),
            _ => (),
        }

        if s.starts_with(|c: char| !c.is_ascii_alphanumeric()) {
            let modifier = ArrayDescDelim::from_byte(s.as_bytes()[0])?;
            let name = interned.intern_str(&s[1..]);

            Ok(Some(Self {
                modifier: Some(modifier),
                name,
            }))
        } else {
            Ok(Some(Self {
                modifier: None,
                name: interned.intern_str(s),
            }))
        }
    }
}
