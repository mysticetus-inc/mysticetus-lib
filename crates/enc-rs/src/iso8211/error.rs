#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
use std::borrow::Cow;
use std::{fmt, io};

use thiserror::Error;

use super::descriptor::field_controls::FieldControlError;
use super::descriptor::format_controls::FormatControlError;
use super::leader::LeaderError;
use crate::utils::InvalidDigitByte;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StaticErr(pub Cow<'static, str>);

impl From<&'static str> for StaticErr {
    fn from(err: &'static str) -> Self {
        Self(Cow::Borrowed(err))
    }
}
impl From<String> for StaticErr {
    fn from(err: String) -> Self {
        Self(Cow::Owned(err))
    }
}

impl fmt::Display for StaticErr {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for StaticErr {}

#[derive(Debug, Error)]
pub enum Iso8211ErrorKind {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Misc(#[from] StaticErr),
    #[error("expected a data descriptive leader, found a data leader instead")]
    UnexpectedDataLeader,
    #[error("expected a data leader, found a data descriptive leader instead")]
    UnexpectedDataDescLeader,
    #[error(transparent)]
    InvalidDigitByte(#[from] InvalidDigitByte),
    #[error(transparent)]
    InvalidLeader(#[from] LeaderError),
    #[error(transparent)]
    InvalidFormatControl(#[from] FormatControlError),
    #[error(transparent)]
    InvalidFieldControl(#[from] FieldControlError),
    #[error("not terminated with a Field Terminator (30/0x1E), found {byte:?}")]
    FieldNotTerminated { byte: Option<u8> },
    #[error("invalid data type: {}", *byte as char)]
    InvalidDataType { byte: u8 },
    #[error("invalid data cardinality: {}", *byte as char)]
    InvalidCardinality { byte: u8 },
    #[error(transparent)]
    ParseError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    FromInvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("unexpected byte found: {found} (expected {expected})")]
    UnexpectedByte { found: u8, expected: u8 },
    #[error("invalid delimiter found while parsing {parsing}: {found}")]
    InvalidDelimiter { found: u8, parsing: &'static str },
    #[error("invalid set of preorder pairs")]
    InvalidPreOrderPairs,
}

#[derive(Debug)]
pub struct Iso8211Error {
    kind: Iso8211ErrorKind,
    #[cfg(feature = "backtrace")]
    backtrace: Backtrace,
}

impl fmt::Display for Iso8211Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;

        #[cfg(feature = "backtrace")]
        write!(f, "\n\n-------BACKTRACE--------\n{}", self.backtrace)?;

        Ok(())
    }
}

impl From<String> for Iso8211Error {
    fn from(misc: String) -> Self {
        Self::misc(misc)
    }
}

impl From<&'static str> for Iso8211Error {
    fn from(misc: &'static str) -> Self {
        Self::misc(misc)
    }
}

impl std::error::Error for Iso8211Error {}

impl<K> From<K> for Iso8211Error
where
    K: Into<Iso8211ErrorKind>,
{
    fn from(kind: K) -> Self {
        Self {
            kind: kind.into(),
            #[cfg(feature = "backtrace")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl Iso8211Error {
    pub(crate) fn eof<M>(message: M) -> Self
    where
        M: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Iso8211ErrorKind::Io(io::Error::new(io::ErrorKind::UnexpectedEof, message.into())).into()
    }

    pub(crate) fn misc_disp<M: fmt::Display>(message: M) -> Self {
        Iso8211ErrorKind::Misc(StaticErr(message.to_string().into())).into()
    }

    pub(crate) fn misc<M: Into<Cow<'static, str>>>(message: M) -> Self {
        Iso8211ErrorKind::Misc(StaticErr(message.into())).into()
    }

    pub(crate) fn unexpected_byte(found: u8, expected: u8) -> Self {
        Iso8211ErrorKind::UnexpectedByte { found, expected }.into()
    }
}

/// Helper impl to assemble an [`io::Error`].
impl From<(io::ErrorKind, &'static str)> for Iso8211Error {
    fn from(tup: (io::ErrorKind, &'static str)) -> Self {
        Iso8211ErrorKind::Io(io::Error::new(tup.0, StaticErr(Cow::Borrowed(tup.1)))).into()
    }
}
