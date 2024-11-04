use std::borrow::Cow;
use std::fmt;

use crate::Value;
use crate::column::InvalidColumnIndex;
use crate::convert::SpannerEncode;
use crate::pool::SessionError;
use crate::ty::SpannerType;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Auth(#[from] gcp_auth_channel::GcpAuthError),
    #[error(transparent)]
    Transport(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("session has been previously deleted")]
    SessionDeleted,
    #[error("transaction was still contended after mulitple retries")]
    TransactionContention,
    #[error("resultset was missing the required metadata")]
    MissingResultMetadata,
    #[error("unexpected number of columns recieved, expected {expected}, found {found}")]
    MismatchedColumnCount { expected: usize, found: usize },
    #[error(transparent)]
    InvalidColumnIndex(#[from] InvalidColumnIndex),
    #[error(transparent)]
    SessionError(#[from] SessionError),
    #[error(transparent)]
    Convert(#[from] ConvertError),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    Misc(#[from] anyhow::Error),
}

const _: () = {
    const fn assert_debug_send_sync_static<T>()
    where
        T: fmt::Debug + Send + Sync + 'static,
    {
    }

    assert_debug_send_sync_static::<Error>();
    assert_debug_send_sync_static::<ConvertError>();
};

impl From<longrunning::Error> for Error {
    fn from(value: longrunning::Error) -> Self {
        match value {
            longrunning::Error::Status(status) => Self::Status(status),
            longrunning::Error::Decode(decode) => {
                Self::Status(tonic::Status::new(tonic::Code::Unknown, decode.to_string()))
            }
        }
    }
}

pub struct ConvertError {
    inner: ConvertErrorInner,
}

impl ConvertError {
    pub fn column<C>(self, col: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        use ConvertErrorInner::*;
        let col = col.into();
        match self.inner {
            MissingTypeInfo(missing) => Self {
                inner: MissingTypeInfo(missing.column(col)),
            },
            From(from) => Self {
                inner: From(from.column(col)),
            },
            Into(into) => Self {
                inner: Into(into.column(col)),
            },
        }
    }
}

impl<T> From<T> for ConvertError
where
    T: Into<ConvertErrorInner>,
{
    #[inline]
    fn from(value: T) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl fmt::Debug for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ConvertError").field(&self.inner).finish()
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl std::error::Error for ConvertError {}

#[derive(Debug)]
pub struct MissingTypeInfo {
    found: Option<protos::spanner::Type>,
    column: Option<Cow<'static, str>>,
}

impl From<MissingTypeInfo> for Error {
    fn from(value: MissingTypeInfo) -> Self {
        Self::Convert(ConvertError::from(value))
    }
}

impl MissingTypeInfo {
    pub(crate) fn missing() -> Self {
        Self {
            found: None,
            column: None,
        }
    }

    pub fn column<C>(mut self, col: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        self.column = Some(col.into());
        self
    }

    pub(crate) fn invalid(ty: protos::spanner::Type) -> Self {
        Self {
            found: Some(ty),
            column: None,
        }
    }
}

impl fmt::Display for MissingTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref col) = self.column {
            f.write_str("'")?;
            f.write_str(col)?;
            f.write_str("': ")?;
        }

        if self.found.is_some() {
            f.write_str("invalid type information recieved")
        } else {
            f.write_str("missing type information, can't infer types properly")
        }
    }
}

impl std::error::Error for MissingTypeInfo {}

#[derive(Debug, thiserror::Error)]
enum ConvertErrorInner {
    #[error(transparent)]
    MissingTypeInfo(#[from] MissingTypeInfo),
    #[error(transparent)]
    Into(#[from] IntoError),
    #[error(transparent)]
    From(#[from] FromError),
}

#[derive(Debug)]
pub struct IntoError {
    info: TypeErrorInfo<Box<dyn fmt::Debug + Send + Sync>>,
    reason: Option<Cow<'static, str>>,
    column: Option<Cow<'static, str>>,
}

impl serde::ser::Error for IntoError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        IntoError {
            info: TypeErrorInfo::Error(anyhow::anyhow!("{msg}")),
            reason: None,
            column: None,
        }
    }
}

impl serde::ser::Error for ConvertError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        ConvertError::from(IntoError::custom(msg))
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Convert(ConvertError::from(IntoError::custom(msg)))
    }
}

impl fmt::Display for IntoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref col) = self.column {
            f.write_str("'")?;
            f.write_str(col)?;
            f.write_str("': ")?;
        }

        match self.info.as_inner_error() {
            Some(error) => fmt::Display::fmt(error, f)?,
            None => f.write_str("couldn't convert into a spanner compatible value")?,
        }

        if let Some(ref reason) = self.reason {
            f.write_str(", ")?;
            f.write_str(reason)?;
        }

        Ok(())
    }
}

impl std::error::Error for IntoError {}

#[derive(Debug)]
pub struct FromError {
    info: TypeErrorInfo<Value>,
    expected: Option<Cow<'static, crate::Type>>,
    column: Option<Cow<'static, str>>,
}

impl fmt::Display for FromError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref col) = self.column {
            f.write_str("'")?;
            f.write_str(col)?;
            f.write_str("': ")?;
        }

        match self.info.as_inner_error() {
            Some(error) => fmt::Display::fmt(error, f)?,
            None => f.write_str("couldn't convert from a spanner compatible value")?,
        }

        if let Some(ref exp) = self.expected {
            f.write_str(", expected ")?;
            <crate::Type as fmt::Display>::fmt(&exp, f)?;
        }
        Ok(())
    }
}

impl std::error::Error for FromError {}

impl serde::de::Error for FromError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        FromError {
            info: TypeErrorInfo::Error(anyhow::anyhow!("{msg}")),
            column: None,
            expected: None,
        }
    }
}

impl serde::de::Error for ConvertError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        ConvertError::from(FromError::custom(msg))
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Convert(ConvertError::from(FromError::custom(msg)))
    }
}

impl From<tonic::transport::Error> for Error {
    #[inline]
    fn from(value: tonic::transport::Error) -> Self {
        Self::Transport(Box::new(value))
    }
}

#[derive(Debug)]
pub enum TypeErrorInfo<V> {
    Value {
        value: V,
        error: Option<anyhow::Error>,
    },
    Error(anyhow::Error),
}

impl<V> TypeErrorInfo<V> {
    pub fn as_inner_error(&self) -> Option<&anyhow::Error> {
        match self {
            Self::Value { error, .. } => error.as_ref(),
            Self::Error(err) => Some(err),
        }
    }
}

// Using 'impl XXX' instead of generics here, since we only want 1 generic param to make the
// requirement to include the destination type via `from_XXXXX::<Self>` easier.
impl FromError {
    pub fn from_value<T: SpannerEncode>(value: impl Into<Value>) -> Self {
        Self {
            info: TypeErrorInfo::Value {
                value: value.into(),
                error: None,
            },
            expected: Some(Cow::Borrowed(&<T::SpannerType as SpannerType>::TYPE)),
            column: None,
        }
    }

    pub fn with_type<T: SpannerEncode>(self) -> Self {
        self.replace_type(Cow::Borrowed(<T::SpannerType as SpannerType>::TYPE))
    }

    pub fn replace_type<C>(mut self, replacement_ty: C) -> Self
    where
        C: Into<Cow<'static, crate::Type>>,
    {
        self.expected = Some(replacement_ty.into());
        self
    }

    pub fn column<C>(mut self, col: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        self.column = Some(col.into());
        self
    }

    pub fn from_value_and_error<T: SpannerEncode>(
        value: impl Into<Value>,
        error: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            info: TypeErrorInfo::Value {
                value: value.into(),
                error: Some(anyhow::anyhow!(error)),
            },
            expected: Some(Cow::Borrowed(<T::SpannerType as SpannerType>::TYPE)),
            column: None,
        }
    }

    pub fn from_error<T: SpannerEncode>(
        error: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            info: TypeErrorInfo::Error(anyhow::anyhow!(error)),
            expected: Some(Cow::Borrowed(<T::SpannerType as SpannerType>::TYPE)),
            column: None,
        }
    }

    pub fn from_anyhow<T: SpannerEncode>(error: anyhow::Error) -> Self {
        Self {
            info: TypeErrorInfo::Error(error),
            expected: Some(Cow::Borrowed(<T::SpannerType as SpannerType>::TYPE)),
            column: None,
        }
    }

    pub fn from_value_and_anyhow<T: SpannerEncode>(
        value: impl Into<Value>,
        error: anyhow::Error,
    ) -> Self {
        Self {
            info: TypeErrorInfo::Value {
                value: value.into(),
                error: Some(error),
            },
            expected: Some(Cow::Borrowed(<T::SpannerType as SpannerType>::TYPE)),
            column: None,
        }
    }
}

impl IntoError {
    pub fn from_value<V>(value: V) -> Self
    where
        V: fmt::Debug + Send + Sync + 'static,
    {
        Self {
            info: TypeErrorInfo::Value {
                value: Box::new(value),
                error: None,
            },
            reason: None,
            column: None,
        }
    }

    pub fn column<C>(mut self, col: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        self.column = Some(col.into());
        self
    }

    pub fn reason<T>(mut self, reason: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.reason = Some(reason.into());
        self
    }

    pub fn from_value_and_error<V, E>(value: V, error: E) -> Self
    where
        V: fmt::Debug + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            info: TypeErrorInfo::Value {
                value: Box::new(value),
                error: Some(anyhow::anyhow!(error)),
            },
            reason: None,
            column: None,
        }
    }

    pub fn from_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            info: TypeErrorInfo::Error(anyhow::anyhow!(error)),
            reason: None,
            column: None,
        }
    }

    pub fn from_anyhow(error: anyhow::Error) -> Self {
        Self {
            info: TypeErrorInfo::Error(error),
            reason: None,
            column: None,
        }
    }

    pub fn from_value_and_anyhow<V>(value: V, error: anyhow::Error) -> Self
    where
        V: fmt::Debug + Send + Sync + 'static,
    {
        Self {
            info: TypeErrorInfo::Value {
                value: Box::new(value),
                error: Some(error),
            },
            reason: None,
            column: None,
        }
    }
}

macro_rules! impl_convert_from_infallible {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<std::convert::Infallible> for $t {
                fn from(value: std::convert::Infallible) -> Self {
                    match value {}
                }
            }
        )*
    };
}

impl_convert_from_infallible!(Error, ConvertError, FromError, IntoError, MissingTypeInfo);

/// since [`gcp_auth_channel::Error`] shares many variants with errors encountered, here, this
/// unpacks the variants to avoid nesting.
impl From<gcp_auth_channel::Error> for Error {
    #[inline]
    fn from(err: gcp_auth_channel::Error) -> Self {
        match err {
            gcp_auth_channel::Error::Auth(auth) => Self::Auth(auth),
            gcp_auth_channel::Error::Transport(transport) => Self::Transport(transport),
            gcp_auth_channel::Error::InvalidHeaderName(name) => Self::InvalidHeaderName(name),
            gcp_auth_channel::Error::InvalidHeaderValue(value) => Self::InvalidHeaderValue(value),
        }
    }
}
