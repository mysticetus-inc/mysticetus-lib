const GOOG_REQUEST_PARAMS: http::HeaderName =
    http::HeaderName::from_static("x-goog-request-params");

#[derive(Clone)]
pub struct GoogRequestParam<Svc> {
    svc: Svc,
    value: http::HeaderValue,
}

impl<Svc: std::fmt::Debug> std::fmt::Debug for GoogRequestParam<Svc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogRequestParam")
            .field("svc", &self.svc)
            .field("x-goog-request-params", &self.value)
            .finish()
    }
}

impl<Svc> GoogRequestParam<Svc> {
    pub fn new(svc: Svc, value: http::HeaderValue) -> Self {
        Self { svc, value }
    }

    pub fn new_parse<V>(svc: Svc, value: V) -> Result<Self, V::Error>
    where
        V: TryInto<http::HeaderValue>,
    {
        let value = value.try_into()?;
        Ok(Self::new(svc, value))
    }
}

crate::util::impl_service_for_wrapper_and_ref! {
    GoogRequestParam::<Svc>::svc {
        call: |self, req: Req| req.headers_mut().insert(GOOG_REQUEST_PARAMS, self.value.clone())
    }
}
