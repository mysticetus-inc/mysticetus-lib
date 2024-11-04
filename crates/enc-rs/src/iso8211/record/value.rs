use std::io::Read;

use intern::InternedStr;
use intern::local::LocalInterner;

use crate::Iso8211Reader;
use crate::complex::Complex;
use crate::iso8211::Iso8211Error;
use crate::iso8211::descriptor::format_controls::FormatControl;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    String(InternedStr),
    Float(f64),
    Bytes(Vec<u8>),
    Int(isize),
    Uint(usize),
    Complex(Complex),
    Array(Vec<Value>),
}

impl Value {
    pub(crate) fn read_from<R: Read>(
        reader: &mut Iso8211Reader<R>,
        interner: &mut LocalInterner,
        fmt_ctrl: &FormatControl,
    ) -> Result<Self, Iso8211Error> {
        match fmt_ctrl {
            FormatControl::Numeric(numeric) => numeric.read_value(reader),
            FormatControl::BitString(bs) => bs.read_value(reader, interner).map(Self::String),
            FormatControl::Union(un) => todo!("FormatControl::Union({un:#?})"),
        }
    }
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        match self {
            Self::Float(f) => serializer.serialize_f64(*f),
            Self::String(s) => serializer.serialize_str(s),
            Self::Bytes(b) => serializer.serialize_bytes(b),
            Self::Int(int) => serializer.serialize_i64(*int as i64),
            Self::Uint(uint) => serializer.serialize_u64(*uint as u64),
            Self::Complex(complex) => complex.serialize(serializer),
            Self::Array(array) => {
                let mut seq = serializer.serialize_seq(Some(array.len()))?;
                for elem in array {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}
