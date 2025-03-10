#[derive(Debug, thiserror::Error)]
pub enum Error<S = super::StdError> {
    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Stream(S),
}

impl<S: std::error::Error> From<std::convert::Infallible> for Error<S> {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

impl<S: std::error::Error> serde::de::Error for Error<S> {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Serde(serde::de::Error::custom(msg))
    }
}
