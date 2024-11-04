use axum::response::IntoResponse;

pub struct ResponseError(pub(crate) anyhow::Error);

impl Clone for ResponseError {
    fn clone(&self) -> Self {
        Self(anyhow::anyhow!("{}", self.0))
    }
}

impl ResponseError {
    pub fn new<E>(error: E) -> Self
    where
        anyhow::Error: From<E>,
    {
        Self(anyhow::Error::from(error))
    }
}

impl From<anyhow::Error> for ResponseError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl axum::response::IntoResponseParts for ResponseError {
    type Error = std::convert::Infallible;

    fn into_response_parts(
        self,
        mut res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        res.extensions_mut().insert(self);
        Ok(res)
    }
}

#[repr(transparent)]
pub struct TraceError<E>(pub E);

impl<E> From<E> for TraceError<E> {
    #[inline(always)]
    fn from(value: E) -> Self {
        Self(value)
    }
}

impl<E> IntoResponse for TraceError<E>
where
    E: std::error::Error + Send + Sync + 'static,
    for<'a> &'a E: IntoResponse,
{
    #[inline]
    fn into_response(self) -> axum::response::Response {
        let mut resp = <&E as IntoResponse>::into_response(&self.0);
        resp.extensions_mut().insert(ResponseError::new(self.0));
        resp
    }
}

macro_rules! response_error {
    (
        $(#[$attr:meta])*
        $v:vis enum
        $name:ident { $($(#[$field_attr:meta])* $variant:ident $($rest:tt)*)* }
    ) => {};
}
