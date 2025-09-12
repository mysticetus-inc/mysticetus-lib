pub mod goog_param;
pub mod headers;

pub use goog_param::GoogRequestParam;

#[derive(Debug, Clone)]
pub struct AttachHeader<Svc> {
    svc: Svc,
    name: http::HeaderName,
    value: http::HeaderValue,
}

#[derive(Debug, Clone)]
pub struct AttachHeaderLayer {
    name: http::HeaderName,
    value: http::HeaderValue,
}

impl AttachHeaderLayer {
    pub fn new(name: http::HeaderName, value: http::HeaderValue) -> Self {
        Self { name, value }
    }

    /// Identical to calling [tower::Layer::<Svc>::layer], but avoids
    /// an extra clone.
    pub fn into_service<Svc>(self, svc: Svc) -> AttachHeader<Svc> {
        AttachHeader {
            svc,
            name: self.name,
            value: self.value,
        }
    }

    pub fn new_parse<V>(name: http::HeaderName, value: V) -> Result<Self, V::Error>
    where
        V: TryInto<http::HeaderValue>,
    {
        Ok(Self {
            name,
            value: value.try_into()?,
        })
    }
}

impl<Svc> tower::Layer<Svc> for AttachHeaderLayer {
    type Service = AttachHeader<Svc>;

    fn layer(&self, svc: Svc) -> Self::Service {
        AttachHeader {
            svc,
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

impl<Svc> AttachHeader<Svc> {
    pub fn new(svc: Svc, name: http::HeaderName, value: http::HeaderValue) -> Self {
        Self { svc, name, value }
    }

    pub fn new_parse<V>(svc: Svc, name: http::HeaderName, value: V) -> Result<Self, V::Error>
    where
        V: TryInto<http::HeaderValue>,
    {
        Ok(Self {
            svc,
            name,
            value: value.try_into()?,
        })
    }

    pub fn value(&self) -> &http::HeaderValue {
        &self.value
    }

    pub fn name(&self) -> &http::HeaderName {
        &self.name
    }
}

crate::util::impl_service_for_wrapper_and_ref! {
    AttachHeader::<Svc>::svc {
        call: |self, req: Req| req.headers_mut().insert(self.name.clone(), self.value.clone())
    }
}
