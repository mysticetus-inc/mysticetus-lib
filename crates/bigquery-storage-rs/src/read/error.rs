use std::fmt;

#[derive(Debug, thiserror::Error)]
pub struct DeserializeError {
    path: Option<path_aware_serde::Path>,
    error: ErrorInner,
}

#[derive(Debug, thiserror::Error)]
enum ErrorInner {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Avro(#[from] apache_avro::Error),
    #[error(transparent)]
    InvalidTimestamp(#[from] timestamp::Error),
    #[error("{0}")]
    Misc(Box<str>),
}

impl From<timestamp::error::ConvertError> for DeserializeError {
    fn from(value: timestamp::error::ConvertError) -> Self {
        ErrorInner::InvalidTimestamp(value.into()).into()
    }
}

impl From<path_aware_serde::Error<Self>> for DeserializeError {
    fn from(value: path_aware_serde::Error<Self>) -> Self {
        let (mut wrapped, path) = value.into_inner();
        if path.is_some() {
            wrapped.path = path;
        }
        wrapped
    }
}

impl<E> From<E> for DeserializeError
where
    ErrorInner: From<E>,
{
    #[inline]
    fn from(value: E) -> Self {
        Self {
            path: None,
            error: ErrorInner::from(value),
        }
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path {
            Some(ref path) => write!(f, "{} at `{path}`", self.error),
            None => write!(f, "{}", self.error),
        }
    }
}

impl serde::de::Error for DeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            path: None,
            error: ErrorInner::Misc(msg.to_string().into_boxed_str()),
        }
    }
}
