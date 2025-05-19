//! Possible error types encountered while parsing, deserializing or performing math that can
//! overflow.

use std::fmt;

use serde::de::{self, Unexpected};

use crate::Unit;

/// Enum with the rust number types. Used to debug cast errors in [`OverflowType::Cast`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Num {
    /// Represents [`i8`]
    I8,
    /// Represents [`i16`]
    I16,
    /// Represents [`i32`]
    I32,
    /// Represents [`i64`]
    I64,
    /// Represents [`i128`]
    I128,
    /// Represents [`u8`]
    U8,
    /// Represents [`u16`]
    U16,
    /// Represents [`u32`]
    U32,
    /// Represents [`u64`]
    U64,
    /// Represents [`u128`]
    U128,
    /// Represents [`f32`]
    F32,
    /// Represents [`f64`]
    F64,
}

impl fmt::Display for Num {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I8 => write!(formatter, "i8"),
            Self::I16 => write!(formatter, "i16"),
            Self::I32 => write!(formatter, "i32"),
            Self::I64 => write!(formatter, "i64"),
            Self::I128 => write!(formatter, "i128"),
            Self::U8 => write!(formatter, "u8"),
            Self::U16 => write!(formatter, "u16"),
            Self::U32 => write!(formatter, "u32"),
            Self::U64 => write!(formatter, "u64"),
            Self::U128 => write!(formatter, "u128"),
            Self::F32 => write!(formatter, "f32"),
            Self::F64 => write!(formatter, "f64"),
        }
    }
}

/// Gives information on why a overflow occured for debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowType {
    /// If the overflow was the result of casting to a different data type.
    Cast {
        /// The numeric type that we tried to cast __from__
        from: Num,
        /// The numeric type that we tried to cast __to__
        to: Num,
    },
    /// If the overflow was caused by converting from one unit to another.
    Unit {
        /// The unit that we tried to convert __from__. If [`None`], we converted from a
        /// unit that is not fully supported across the board, (i.e MS Ticks).
        from: Option<Unit>,
        /// The unit that we tried to convert __to__.
        to: Unit,
        /// The numeric data type that we tried to perform the unit conversion with.
        ty: Num,
    },
}

impl From<(Num, Num)> for OverflowType {
    fn from(pair: (Num, Num)) -> Self {
        Self::Cast {
            from: pair.0,
            to: pair.1,
        }
    }
}

impl From<(Option<Unit>, Unit, Num)> for OverflowType {
    fn from(tup: (Option<Unit>, Unit, Num)) -> Self {
        Self::Unit {
            from: tup.0,
            to: tup.1,
            ty: tup.2,
        }
    }
}

impl fmt::Display for OverflowType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Cast { from, to } => {
                write!(formatter, "casting from {from} to {to} caused an overflow")
            }
            Self::Unit { from, to, ty } => match from {
                Some(from) => {
                    write!(
                        formatter,
                        "converting from {from} to {to} (as a {ty}) caused an overflow"
                    )
                }
                _ => write!(
                    formatter,
                    "converting to {to} (as a {ty}) caused an overflow"
                ),
            },
        }
    }
}

impl<O> From<O> for ConvertError
where
    O: Into<OverflowType>,
{
    fn from(overflow: O) -> Self {
        Self::Overflow(overflow.into())
    }
}

/// A general error for a failed conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertError {
    /// Indicates a conversion ran into an integer overflow
    Overflow(OverflowType),
    /// Indicates that a conversion is out of range for valid [`Timestamp`]'s
    ///
    /// [`Timestamp`]: [`crate::Timestamp`]
    OutOfRange,
    /// Indicates that a floating point value was either 'NaN' or '+/- Inf'
    NotFinite,
}

impl ConvertError {
    /// Since [`From::from`] isn't const, the shortcuts don't quite work. This provides roughly
    /// the same utility while being const.
    pub(crate) const fn overflow_unit(from: Option<Unit>, to: Unit, ty: Num) -> Self {
        Self::Overflow(OverflowType::Unit { from, to, ty })
    }

    #[allow(dead_code)]
    /// Similar const helper as [`Self::overflow_unit`]
    pub(crate) const fn overflow_cast(from: Num, to: Num) -> Self {
        Self::Overflow(OverflowType::Cast { from, to })
    }

    pub(crate) fn into_serde<E>(self, unexpected: Unexpected) -> E
    where
        E: de::Error,
    {
        match self {
            Self::Overflow(overflow) => {
                de::Error::invalid_type(unexpected, &overflow.to_string().as_ref())
            }
            Self::OutOfRange => {
                de::Error::invalid_value(unexpected, &"a timestamp within 0001-01-01 -> 9999-12-30")
            }
            Self::NotFinite => {
                de::Error::invalid_value(unexpected, &"floating point value is not finite")
            }
        }
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Overflow(overflow) => write!(formatter, "{overflow}"),
            Self::OutOfRange => write!(formatter, "value is out of range for a 'Timestamp'"),
            Self::NotFinite => write!(formatter, "floating point value is not finite"),
        }
    }
}

impl std::error::Error for ConvertError {}

/// Error types that can be encountered
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Datetime string parsing error
    #[error("{0}")]
    Parse(#[from] chrono::ParseError),
    /// In-Progress replacement for [Error::Parse].
    #[error(transparent)]
    ParseParts(#[from] crate::parse::ParseError),
    /// Errors parsing a float from a string.
    #[error("error parsing float string: {0}")]
    FloatParse(#[from] std::num::ParseFloatError),
    /// Conversion errors (casting/unit conversion/out-of-range)
    #[error("{0}")]
    Convert(#[from] ConvertError),
    /// Error detailing a value that is out of range of the given unit.
    #[error("{0}")]
    ComponentRange(#[from] time::error::ComponentRange),
    /// Custom deserialization errors
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Formats 'self' as an arbitrary [`serde::de::Error`], given the invalid
    /// value we tried to parse from.
    pub fn into_de_error<E>(self, unexpected: Unexpected<'_>) -> E
    where
        E: de::Error,
    {
        match self {
            Self::Parse(parse) => de::Error::invalid_value(unexpected, &parse.to_string().as_ref()),
            Self::ParseParts(parts) => de::Error::custom(parts),
            Self::FloatParse(float_err) => de::Error::custom(format!(
                "could not parse float string: '{unexpected}': {float_err}",
            )),
            Self::ComponentRange(range) => de::Error::invalid_value(unexpected, &range),
            Self::Convert(convert) => convert.into_serde(unexpected),
            Self::Custom(string) => de::Error::invalid_value(unexpected, &&*string),
        }
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}
