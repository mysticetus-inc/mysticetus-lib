use crate::http_svc::HttpRequest;

#[derive(Debug, Clone)]
pub struct AttachHeaders<Svc, Pairs> {
    svc: Svc,
    pairs: Pairs,
}

#[derive(Debug, Clone)]
pub struct AttachHeadersLayer<Pairs> {
    pairs: Pairs,
}

impl<Pairs> AttachHeadersLayer<Pairs> {
    pub fn new(pairs: Pairs) -> Self {
        Self { pairs }
    }

    pub fn new_parse<K, V, Pair>(
        pairs: impl IntoIterator<Item = (K, V)>,
        mut make_pair: impl FnMut(http::HeaderName, http::HeaderValue) -> Pair,
    ) -> Result<
        Self,
        Result<<http::HeaderName as TryFrom<K>>::Error, <http::HeaderValue as TryFrom<V>>::Error>,
    >
    where
        Pairs: FromIterator<Pair>,
        http::HeaderName: TryFrom<K>,
        http::HeaderValue: TryFrom<V>,
    {
        fn parse_pair<K, V, Pair>(
            k: K,
            v: V,
            make_pair: impl FnOnce(http::HeaderName, http::HeaderValue) -> Pair,
        ) -> Result<
            Pair,
            Result<
                <http::HeaderName as TryFrom<K>>::Error,
                <http::HeaderValue as TryFrom<V>>::Error,
            >,
        >
        where
            http::HeaderName: TryFrom<K>,
            http::HeaderValue: TryFrom<V>,
        {
            let name = http::HeaderName::try_from(k).map_err(Ok)?;
            let value = http::HeaderValue::try_from(v).map_err(Err)?;
            Ok(make_pair(name, value))
        }

        pairs
            .into_iter()
            .map(|(k, v)| parse_pair(k, v, &mut make_pair))
            .collect::<Result<Pairs, _>>()
            .map(Self::new)
    }

    /// Identical to calling [tower::Layer::<Svc>::layer], but avoids
    /// an extra clone.
    pub fn into_service<Svc>(self, svc: Svc) -> AttachHeaders<Svc, Pairs> {
        AttachHeaders {
            svc,
            pairs: self.pairs,
        }
    }
}

impl<Svc, Pairs: Clone> tower::Layer<Svc> for AttachHeadersLayer<Pairs> {
    type Service = AttachHeaders<Svc, Pairs>;

    fn layer(&self, svc: Svc) -> Self::Service {
        AttachHeaders {
            svc,
            pairs: self.pairs.clone(),
        }
    }
}

impl<Svc, Pairs> AttachHeaders<Svc, Pairs> {
    pub fn new(svc: Svc, pairs: Pairs) -> Self {
        Self { svc, pairs }
    }
}

fn insert_headers<'a, Pairs>(pairs: &'a Pairs, dst: &mut http::HeaderMap)
where
    &'a Pairs: IntoIterator<Item: HeaderPair<'a>>,
{
    for pair in pairs {
        dst.insert(pair.name().clone(), pair.value().clone());
    }
}

pub trait HeaderPair<'a> {
    fn name(&self) -> &'a http::HeaderName;

    fn value(&self) -> &'a http::HeaderValue;
}

impl<'a, Pair: ?Sized> HeaderPair<'a> for &'a Pair
where
    Pair: HeaderPair<'a>,
{
    #[inline]
    fn name(&self) -> &'a http::HeaderName {
        Pair::name(self)
    }

    #[inline]
    fn value(&self) -> &'a http::HeaderValue {
        Pair::value(self)
    }
}

impl<'a> HeaderPair<'a> for (&'a http::HeaderName, &'a http::HeaderValue) {
    #[inline]
    fn name(&self) -> &'a http::HeaderName {
        self.0
    }

    #[inline]
    fn value(&self) -> &'a http::HeaderValue {
        self.1
    }
}

impl<'a> HeaderPair<'a> for &'a (http::HeaderName, http::HeaderValue) {
    #[inline]
    fn name(&self) -> &'a http::HeaderName {
        &self.0
    }

    #[inline]
    fn value(&self) -> &'a http::HeaderValue {
        &self.1
    }
}

impl<Svc, Pairs, Req> tower::Service<Req> for AttachHeaders<Svc, Pairs>
where
    for<'a> &'a Pairs: IntoIterator<Item: HeaderPair<'a>>,
    Svc: tower::Service<Req>,
    Req: HttpRequest,
{
    type Error = Svc::Error;
    type Future = Svc::Future;
    type Response = Svc::Response;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, mut req: Req) -> Self::Future {
        insert_headers(&self.pairs, req.headers_mut());
        self.svc.call(req)
    }
}

impl<'a, Svc, Pairs, Req> tower::Service<Req> for &'a AttachHeaders<Svc, Pairs>
where
    for<'b> &'b Pairs: IntoIterator<Item: HeaderPair<'b>>,
    &'a Svc: tower::Service<Req>,
    Req: HttpRequest,
{
    type Error = <&'a Svc as tower::Service<Req>>::Error;
    type Future = <&'a Svc as tower::Service<Req>>::Future;
    type Response = <&'a Svc as tower::Service<Req>>::Response;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        (&mut &self.svc).poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, mut req: Req) -> Self::Future {
        insert_headers(&self.pairs, req.headers_mut());
        (&mut &self.svc).call(req)
    }
}
