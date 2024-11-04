use std::io::Read;

use intern::local::LocalInterner;
use intern::{InternedStr, Interner};

use super::field_controls::FieldControls;
use crate::complex::Complex;
use crate::iso8211::error::Iso8211Error;
use crate::iso8211::record::value::Value;
use crate::iso8211::terminator;
use crate::utils::{
    InvalidDigitByte, ascii_byte_to_digit, ascii_byte_to_digit_unsafe, byte_enum, chars_to_usize,
};
use crate::{Endianness, FromByte, Iso8211Reader};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum FormatControl {
    BitString(BitString),
    Numeric(Numeric),
    Union(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Width {
    Known(usize),
    Delimiter(u8),
    DelimiterString(InternedStr),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct BitString {
    pub(crate) width: Option<Width>,
    pub(crate) kind: BitStringKind,
}

impl BitString {
    pub fn min_length(&self) -> Option<usize> {
        match &self.width {
            Some(Width::Known(n)) => Some(*n),
            Some(Width::Delimiter(_)) => Some(1),
            Some(Width::DelimiterString(delim)) => Some(delim.len()),
            _ => None,
        }
    }

    pub fn known_length(&self) -> Option<usize> {
        match self.width {
            Some(Width::Known(known)) => Some(known),
            _ => None,
        }
    }

    pub(crate) fn read_value<R: Read>(
        &self,
        reader: &mut Iso8211Reader<R>,
        interner: &mut LocalInterner,
    ) -> Result<InternedStr, Iso8211Error> {
        match self.width {
            Some(Width::Known(bits)) => match self.kind {
                BitStringKind::CharacterData | BitStringKind::BitStringData => {
                    if bits % 8 != 0 {
                        return Err(Iso8211Error::misc(format!(
                            "expected a multiple of 8, found: {bits}"
                        )));
                    }

                    let bytes = reader.read_bytes(bits / 8)?;
                    let s = String::from_utf8_lossy(&bytes[..bytes.len() - 1]);
                    match s {
                        std::borrow::Cow::Owned(o) => Ok(interner.intern_string(o)),
                        std::borrow::Cow::Borrowed(b) => Ok(interner.intern_str(b)),
                    }
                }
                BitStringKind::ExplicitPoint => {
                    Ok(interner.intern_str(reader.read_sized_str(bits)?))
                }
                other => todo!("other bit string: {other:?}"),
            },
            Some(Width::Delimiter(d)) => Ok(interner.intern_str(reader.read_str_until(d)?)),
            Some(Width::DelimiterString(_)) => todo!("no support for delim strings yet"),
            None => match self.kind {
                BitStringKind::CharacterData => {
                    let (s, _) =
                        reader.read_str_until_either(terminator::FIELD, terminator::UNIT)?;

                    Ok(interner.intern_str(s))
                }
                BitStringKind::BitStringData => {
                    let str_len_bytes = reader.read_char_digit()? as usize;
                    println!("{str_len_bytes}");
                    let str_len_bytes = reader.read_bytes(str_len_bytes)?;
                    let str_len = chars_to_usize(str_len_bytes)?;
                    let s = reader.read_sized_str(str_len)?;
                    Ok(interner.intern_str(s))
                }
                other => todo!("other bit string: {other:?}"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, serde::Serialize)]
pub struct Numeric {
    pub(crate) kind: NumericKind,
    pub(crate) endianness: Endianness,
}

const _: () = {
    // the raw info fits in 3 bytes, so this should too
    if std::mem::size_of::<Numeric>() != 3 {
        panic!(concat!("unexpected size for Numeric (in ", file!(), ")"));
    }
};

impl Numeric {
    pub fn size(&self) -> usize {
        match self.kind {
            NumericKind::Int(int) => int.precision.get(),
            NumericKind::Float(prec) => prec.get(),
            NumericKind::Complex(prec) => 2 * prec.get(),
            NumericKind::FixedPoint(prec) => prec.get(),
        }
    }

    pub fn parse(
        s: &str,
        endianness: Endianness,
        _field_controls: &FieldControls,
    ) -> Result<Self, FormatControlError> {
        if s.len() != 3 {
            return Err(FormatControlError::InvalidNumericControl(s.to_owned()));
        }

        let bytes = s.as_bytes();

        let kind = match bytes[1] {
            b'1' => NumericKind::Int(IntFormat {
                precision: IntPrecision::from_byte(bytes[2])?,
                signed: false,
            }),
            b'2' => NumericKind::Int(IntFormat {
                precision: IntPrecision::from_byte(bytes[2])?,
                signed: true,
            }),
            b'3' => NumericKind::FixedPoint(FloatPrecision::from_byte(bytes[2])?),
            b'4' => NumericKind::Float(FloatPrecision::from_byte(bytes[2])?),
            b'5' => NumericKind::Complex(FloatPrecision::from_byte(bytes[2])?),
            _ => return Err(FormatControlError::InvalidNumericControl(s.to_owned())),
        };

        Ok(Self { kind, endianness })
    }

    pub fn read_value<R: Read>(
        &self,
        reader: &mut Iso8211Reader<R>,
    ) -> Result<Value, Iso8211Error> {
        match self.kind {
            NumericKind::Int(int) => {
                if int.signed {
                    int.precision
                        .read_signed(reader, self.endianness)
                        .map(Value::Int)
                } else {
                    int.precision
                        .read_unsigned(reader, self.endianness)
                        .map(Value::Uint)
                }
            }
            NumericKind::FixedPoint(prec) => {
                let half_prec = match prec {
                    FloatPrecision::Four => IntPrecision::Two,
                    FloatPrecision::Eight => IntPrecision::Four,
                };

                let whole_part = half_prec.read_signed(reader, self.endianness)? as f64;
                let mut frac_part = half_prec.read_unsigned(reader, self.endianness)? as f64;
                while frac_part >= 0.0 {
                    frac_part /= 10.0;
                }

                Ok(Value::Float(whole_part + frac_part))
            }
            NumericKind::Float(prec) => prec.read_float(reader, self.endianness).map(Value::Float),
            NumericKind::Complex(prec) => {
                let real = prec.read_float(reader, self.endianness)?;
                let imag = prec.read_float(reader, self.endianness)?;
                Ok(Value::Complex(Complex::new(real, imag)))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(tag = "kind")]
pub enum NumericKind {
    Int(IntFormat),
    FixedPoint(FloatPrecision),
    Float(FloatPrecision),
    Complex(FloatPrecision),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct IntFormat {
    precision: IntPrecision,
    signed: bool,
}

struct FmtCtrlParseIter<'a> {
    iter: std::str::SplitTerminator<'a, char>,
    field_controls: &'a FieldControls,
    interner: &'a mut LocalInterner,
    current: Option<FormatControl>,
    repeat: usize,
}

impl<'a> FmtCtrlParseIter<'a> {
    fn from_reader<R: Read>(
        reader: &'a mut Iso8211Reader<R>,
        interner: &'a mut LocalInterner,
        field_controls: &'a FieldControls,
    ) -> Result<Self, Iso8211Error> {
        let s = reader
            .read_str_until(terminator::FIELD)?
            .trim_start_matches('(')
            .trim_end_matches(')');

        Ok(Self {
            iter: s.split_terminator(','),
            current: None,
            interner,
            field_controls,
            repeat: 0,
        })
    }
}

impl<'a> Iterator for FmtCtrlParseIter<'a> {
    type Item = Result<FormatControl, Iso8211Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.repeat > 0 {
            self.repeat -= 1;

            let next = if self.repeat == 0 {
                self.current.take().unwrap()
            } else {
                self.current.as_ref().unwrap().clone()
            };

            return Some(Ok(next));
        }

        let next_str = self.iter.next()?;

        match FormatControl::parse_single_w_repeat(next_str, self.field_controls, self.interner) {
            Ok((0, fmt)) => return Some(Ok(fmt)),
            Ok((repeats, fmt)) => {
                self.repeat = repeats - 1;
                let next = self.current.insert(fmt).clone();
                return Some(Ok(next));
            }
            Err(err) => return Some(Err(err)),
        }
    }
}

impl FormatControl {
    pub fn min_length(&self) -> Option<usize> {
        match self {
            Self::Numeric(numeric) => Some(numeric.size()),
            Self::BitString(bs) => bs.min_length(),
            Self::Union(un) => {
                let mut sum = 0;
                for item in un {
                    if let Some(min) = item.min_length() {
                        sum += min;
                    }
                }

                Some(sum)
            }
        }
    }

    pub fn known_length(&self) -> Option<usize> {
        match self {
            Self::Numeric(numeric) => Some(numeric.size()),
            Self::BitString(bs) => bs.known_length(),
            Self::Union(un) => {
                let mut sum = 0;
                for item in un {
                    sum += item.known_length()?;
                }

                Some(sum)
            }
        }
    }

    pub fn from_reader<R: Read, F, O>(
        reader: &mut Iso8211Reader<R>,
        field_controls: &FieldControls,
        interner: &mut LocalInterner,
        mut map_fn: F,
    ) -> Result<Vec<O>, Iso8211Error>
    where
        F: FnMut(FormatControl) -> Result<O, Iso8211Error>,
    {
        let s = reader
            .read_str_until(terminator::FIELD)?
            .trim_start_matches('(')
            .trim_end_matches(')');

        let split_iter = s.split_terminator(',');

        let mut dst = Vec::with_capacity(s.len() / 4);

        for component in split_iter {
            let (repeats, fmt) = Self::parse_single_w_repeat(component, field_controls, interner)?;

            // avoid cloning if we only need to insert 1.
            for _ in 0..repeats.saturating_sub(1) {
                let out = map_fn(fmt.clone())?;
                dst.push(out);
            }

            let out = map_fn(fmt)?;
            dst.push(out);
        }

        Ok(dst)
    }

    fn parse_single_w_repeat(
        s: &str,
        field_controls: &FieldControls,
        interner: &mut LocalInterner,
    ) -> Result<(usize, Self), Iso8211Error> {
        let stripped = s.trim_start_matches(|c: char| c.is_ascii_digit());

        let leading_digits_len = s.len() - stripped.len();
        let repeats = match leading_digits_len {
            0 => 1,
            1 => ascii_byte_to_digit(s.as_bytes()[0])? as usize,
            n => s[..n].parse::<usize>()?,
        };

        // check if this is a union, since we don't parse it in parse_single.
        let fmt = if stripped.contains('|') {
            let fmts = stripped
                .split_terminator('|')
                .filter(|s| !s.is_empty())
                .map(|s| Self::parse_single(s, field_controls, interner))
                .collect::<Result<Vec<Self>, FormatControlError>>()?;

            FormatControl::Union(fmts)
        } else {
            Self::parse_single(stripped, field_controls, interner)?
        };

        Ok((repeats, fmt))
    }

    fn parse_single(
        s: &str,
        field_controls: &FieldControls,
        interner: &mut LocalInterner,
    ) -> Result<Self, FormatControlError> {
        if s.is_empty() {
            return Err(FormatControlError::EmptyString);
        }

        // this is the only value not included in BitStringKind that we can use to determine what
        // kind this is, so check it first (and it tends to be fairly common, since its LE
        // digits).
        if s.starts_with('b') {
            let numeric = Numeric::parse(s, Endianness::Little, field_controls)?;
            return Ok(Self::Numeric(numeric));
        }

        let kind = BitStringKind::from_byte(s.as_bytes()[0])?;

        // length of 1 means no length/etc to parse
        if s.len() == 1 {
            return Ok(Self::BitString(BitString { kind, width: None }));
        }

        // if the kind is 'B', it could also be a big endian numeric, but only if the len == 3,
        // and there's no paranthases.
        if kind == BitStringKind::BitStringData && s.len() == 3 && !s.as_bytes().ends_with(b")") {
            let num = Numeric::parse(s, Endianness::Big, field_controls)?;
            return Ok(Self::Numeric(num));
        }

        let width_bytes = s[1..]
            .trim_start_matches('(')
            .trim_end_matches(')')
            .as_bytes();

        let width = if width_bytes.len() == 1 {
            if width_bytes[0].is_ascii_digit() {
                // SAFETY: we just checked above that the byte is indeed an ascii digit.
                let width = unsafe { ascii_byte_to_digit_unsafe(width_bytes[0]) as usize };
                Width::Known(width)
            } else {
                Width::Delimiter(width_bytes[0])
            }
        } else if width_bytes.iter().all(|b| b.is_ascii_digit()) {
            let width = chars_to_usize(width_bytes)?;
            Width::Known(width)
        } else {
            let delim = std::str::from_utf8(width_bytes)?;
            Width::DelimiterString(interner.intern_str(delim))
        };

        Ok(Self::BitString(BitString {
            kind,
            width: Some(width),
        }))
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum BitStringKind {
        CharacterData = b'A',
        ImplicitPoint = b'I',
        ExplicitPoint = b'R',
        ExplicitPointScaled = b'S',
        CharacterModeBitString = b'C',
        BitStringData = b'B',
        UnusedCharacterPositions = b'X',
    }
    from_byte_error(byte: u8, const) -> FormatControlError {
        FormatControlError::InvalidBitStringKind { byte }
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum IntPrecision {
        One = b'1',
        Two = b'2',
        Three = b'3',
        Four = b'4',
    }
    from_byte_error(byte: u8, const) -> FormatControlError {
        FormatControlError::InvalidIntPrecision { byte }
    }
}

impl serde::Serialize for IntPrecision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.get() as u32)
    }
}

impl IntPrecision {
    pub fn get(&self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    pub fn read_signed<R: Read>(
        &self,
        reader: &mut Iso8211Reader<R>,
        endianness: Endianness,
    ) -> Result<isize, Iso8211Error> {
        macro_rules! convert {
            ($r:expr) => {{
                match $r {
                    Ok(int) => Ok(int as isize),
                    Err(err) => Err(err.into()),
                }
            }};
        }

        match self {
            Self::One => convert!(reader.read_i8(endianness)),
            Self::Two => convert!(reader.read_i16(endianness)),
            Self::Four => convert!(reader.read_i32(endianness)),
            Self::Three => {
                let [b1, b2, b3] = *reader.read_array::<3>()?;

                match endianness {
                    Endianness::Little => Ok(i32::from_le_bytes([b1, b2, b3, 0]) as isize),
                    Endianness::Big => Ok(i32::from_be_bytes([0, b1, b2, b3]) as isize),
                }
            }
        }
    }

    pub fn read_unsigned<R: Read>(
        &self,
        reader: &mut Iso8211Reader<R>,
        endianness: Endianness,
    ) -> Result<usize, Iso8211Error> {
        macro_rules! convert {
            ($r:expr) => {{
                match $r {
                    Ok(int) => Ok(int as usize),
                    Err(err) => Err(err.into()),
                }
            }};
        }

        match self {
            Self::One => convert!(reader.read_byte()),
            Self::Two => convert!(reader.read_u16(endianness)),
            Self::Four => convert!(reader.read_u32(endianness)),
            Self::Three => {
                let [b1, b2, b3] = *reader.read_array::<3>()?;

                match endianness {
                    Endianness::Little => Ok(u32::from_le_bytes([b1, b2, b3, 0]) as usize),
                    Endianness::Big => Ok(u32::from_be_bytes([0, b1, b2, b3]) as usize),
                }
            }
        }
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum FloatPrecision {
        Four = b'4',
        Eight = b'8',
    }
    from_byte_error(byte: u8, const) -> FormatControlError {
        FormatControlError::InvalidFloatPrecision { byte }
    }
}

impl serde::Serialize for FloatPrecision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.get() as u32)
    }
}

impl FloatPrecision {
    pub fn get(&self) -> usize {
        match self {
            Self::Four => 4,
            Self::Eight => 8,
        }
    }

    pub fn read_float<R: Read>(
        &self,
        reader: &mut Iso8211Reader<R>,
        endianness: Endianness,
    ) -> Result<f64, Iso8211Error> {
        match self {
            Self::Four => Ok(reader.read_f32(endianness)? as f64),
            Self::Eight => Ok(reader.read_f64(endianness)?),
        }
    }
}

impl crate::FromByte for Endianness {
    type Error = FormatControlError;

    fn from_byte(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            b'b' => Ok(Self::Little),
            b'B' => Ok(Self::Big),
            _ => Err(FormatControlError::InvalidEndianness { byte }),
        }
    }
}

macro_rules! read_int {
    ($t:ty : $endianness:expr, $bytes:expr) => {{
        match $endianness {
            Endianness::Little => <$t>::from_le_bytes($bytes),
            Endianness::Big => <$t>::from_be_bytes($bytes),
        }
    }};
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum FormatControlError {
    #[error("invalid endianness found: {} (expected 'b' or 'B')", *byte as char)]
    InvalidEndianness { byte: u8 },
    #[error("invalid float precision found: {} (expected '4' or '8')", *byte as char)]
    InvalidFloatPrecision { byte: u8 },
    #[error("invalid integer precision found: {} (expected '1', '2', '3' or '4')", *byte as char)]
    InvalidIntPrecision { byte: u8 },
    #[error("invalid bit string kind found: {} (expected 'A', 'I', 'R', 'S', 'C', 'B' or 'X')", *byte as char)]
    InvalidBitStringKind { byte: u8 },
    #[error("invalid numeric format control found: '{0}' (expected 'B##' or 'b##')")]
    InvalidNumericControl(String),
    #[error("format control cannot be parsed from an empty string")]
    EmptyString,
    #[error(transparent)]
    InvalidDigitByte(#[from] InvalidDigitByte),
    #[error("invalid delimiter: {0}")]
    InvalidDelimiter(#[from] std::str::Utf8Error),
}
