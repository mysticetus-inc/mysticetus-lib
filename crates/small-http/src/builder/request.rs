use http::{HeaderMap, HeaderName, HeaderValue};
use hyper::body::Incoming;

use crate::version::DefaultHttpVersion;
use crate::{Client, ConnectionParts, HttpVersion, SendRequest};

pub struct RequestBuilder<'a, B, HttpVers: HttpVersion<B> = DefaultHttpVersion> {
    url: url::Url,
    builder: http::request::Builder,
    parts: &'a mut ConnectionParts<B, HttpVers>,
}

impl<'a, B: 'static, HttpVers: HttpVersion<B>> RequestBuilder<'a, B, HttpVers> {
    pub(crate) fn new(
        parts: &'a mut ConnectionParts<B, HttpVers>,
        base_uri: &url::Url,
        default_headers: &HeaderMap,
    ) -> Self {
        let mut builder = http::request::Builder::new();

        if !default_headers.is_empty() {
            let headers = builder
                .headers_mut()
                .expect("just created, should have no errors");

            for (header, value) in default_headers.iter() {
                headers.append(header, value.clone());
            }
        }

        Self {
            url: base_uri.clone(),
            parts,
            builder,
        }
    }

    pub fn append_path(mut self, path: &str) -> Self {
        let mut segs_mut = self.url.path_segments_mut().unwrap();
        segs_mut.pop_if_empty().push(path);
        drop(segs_mut);
        self
    }

    pub fn query_param(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let mut serializer = self.url.query_pairs_mut();
        serializer
            .append_pair(name.as_ref(), value.as_ref())
            .finish();

        drop(serializer);

        self
    }

    pub fn query_params<K, V>(mut self, params: &[(K, V)]) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut serializer = self.url.query_pairs_mut();

        for (name, value) in params {
            serializer.append_pair(name.as_ref(), value.as_ref());
        }

        serializer.finish();

        drop(serializer);

        self
    }

    pub fn header<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        HeaderName: TryFrom<Name>,
        <HeaderName as TryFrom<Name>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<Value>,
        <HeaderValue as TryFrom<Value>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(name, value);
        self
    }

    #[inline]
    pub async fn get(self, body: B) -> Result<http::Response<Incoming>, crate::Error> {
        self.execute(http::Method::GET, body).await
    }

    #[inline]
    pub async fn post(self, body: B) -> Result<http::Response<Incoming>, crate::Error> {
        self.execute(http::Method::POST, body).await
    }

    async fn execute(
        self,
        method: http::Method,
        body: B,
    ) -> Result<http::Response<Incoming>, crate::Error> {
        let Self {
            url,
            builder,
            parts,
        } = self;

        let request = builder
            .method(method)
            .uri::<String>(url.into())
            .body(body)?;

        println!("request headers: {:#?}", request.headers());

        parts.send_request.ready().await?;

        parts
            .send_request
            .send_request(request)
            .await
            .map_err(crate::Error::Hyper)
    }
}
