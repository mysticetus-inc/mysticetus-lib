//! Builders for [`WithHeader`].
use std::marker::PhantomData;
use std::str::FromStr;

use super::{AuthChannel, KeyValuePair, WithHeader, WithHeaderLayer};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WithHeaderLayerBuilder<Kvp: KeyValuePair, K, V> {
    key: K,
    value: V,
    _marker: PhantomData<Kvp>,
}

impl<Kvp> WithHeaderLayerBuilder<Kvp, (), ()>
where
    Kvp: KeyValuePair,
{
    pub const fn new() -> Self {
        Self {
            key: (),
            value: (),
            _marker: PhantomData,
        }
    }
}

impl<Kvp> Default for WithHeaderLayerBuilder<Kvp, (), ()>
where
    Kvp: KeyValuePair,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Kvp> WithHeaderLayerBuilder<Kvp, (), ()>
where
    Kvp: KeyValuePair,
{
    #[inline]
    pub fn static_key(self, key: &'static str) -> WithHeaderLayerBuilder<Kvp, Kvp::Key, ()> {
        self.key(Kvp::key_from_static(key))
    }

    #[inline]
    pub fn key<K>(self, key: K) -> WithHeaderLayerBuilder<Kvp, Kvp::Key, ()>
    where
        K: Into<Kvp::Key>,
    {
        WithHeaderLayerBuilder {
            key: key.into(),
            value: (),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn parse_key<S>(
        self,
        key: S,
    ) -> Result<WithHeaderLayerBuilder<Kvp, Kvp::Key, ()>, <Kvp::Key as FromStr>::Err>
    where
        S: AsRef<str>,
    {
        let parsed = key.as_ref().parse::<Kvp::Key>()?;
        Ok(self.key(parsed))
    }

    #[inline]
    pub fn try_key<S>(self, key: S) -> Result<WithHeaderLayerBuilder<Kvp, Kvp::Key, ()>, S::Error>
    where
        S: TryInto<Kvp::Key>,
    {
        let parsed = key.try_into()?;
        Ok(self.key(parsed))
    }
}

impl<Kvp> WithHeaderLayerBuilder<Kvp, Kvp::Key, ()>
where
    Kvp: KeyValuePair,
{
    #[inline]
    pub fn static_value(self, value: &'static str) -> WithHeaderLayer<Kvp> {
        self.value(Kvp::value_from_static(value))
    }

    #[inline]
    pub fn value<V>(self, value: V) -> WithHeaderLayer<Kvp>
    where
        V: Into<Kvp::Value>,
    {
        WithHeaderLayer {
            key: self.key,
            value: value.into(),
        }
    }

    #[inline]
    pub fn parse_value<S>(
        self,
        value: S,
    ) -> Result<WithHeaderLayer<Kvp>, <Kvp::Value as FromStr>::Err>
    where
        S: AsRef<str>,
    {
        let parsed = value.as_ref().parse::<Kvp::Value>()?;
        Ok(self.value(parsed))
    }

    #[inline]
    pub fn try_value<V>(
        self,
        value: V,
    ) -> Result<WithHeaderLayer<Kvp>, <V as TryInto<Kvp::Value>>::Error>
    where
        V: TryInto<Kvp::Value>,
    {
        let parsed = value.try_into()?;
        Ok(self.value(parsed))
    }
}

#[derive(Debug, Clone)]
pub struct WithHeaderBuilder<Serv, Kvp: KeyValuePair, K, V> {
    service: AuthChannel<Serv>,
    inner: WithHeaderLayerBuilder<Kvp, K, V>,
}

impl<Kvp, Serv> WithHeaderBuilder<Serv, Kvp, (), ()>
where
    Kvp: KeyValuePair,
{
    #[inline]
    pub const fn from_service(service: AuthChannel<Serv>) -> Self {
        Self {
            service,
            inner: WithHeaderLayerBuilder::new(),
        }
    }

    #[inline]
    pub fn static_key(self, key: &'static str) -> WithHeaderBuilder<Serv, Kvp, Kvp::Key, ()> {
        WithHeaderBuilder {
            service: self.service,
            inner: self.inner.static_key(key),
        }
    }

    #[inline]
    pub fn key<K>(self, key: K) -> WithHeaderBuilder<Serv, Kvp, Kvp::Key, ()>
    where
        K: Into<Kvp::Key>,
    {
        WithHeaderBuilder {
            service: self.service,
            inner: self.inner.key(key),
        }
    }

    #[inline]
    pub fn parse_key<S, E>(self, key: S) -> Result<WithHeaderBuilder<Serv, Kvp, Kvp::Key, ()>, E>
    where
        S: AsRef<str>,
        Kvp::Key: FromStr<Err = E>,
    {
        let inner = self.inner.parse_key(key)?;
        Ok(WithHeaderBuilder {
            inner,
            service: self.service,
        })
    }

    #[inline]
    pub fn try_key<S>(self, key: S) -> Result<WithHeaderBuilder<Serv, Kvp, Kvp::Key, ()>, S::Error>
    where
        S: TryInto<Kvp::Key>,
    {
        let inner = self.inner.try_key(key)?;
        Ok(WithHeaderBuilder {
            inner,
            service: self.service,
        })
    }
}

impl<Kvp, Serv> WithHeaderBuilder<Serv, Kvp, Kvp::Key, ()>
where
    Kvp: KeyValuePair,
{
    #[inline]
    pub fn static_value(self, value: &'static str) -> AuthChannel<WithHeader<Serv, Kvp>> {
        self.inner.static_value(value).layer(self.service)
    }

    #[inline]
    pub fn value<V>(self, value: V) -> AuthChannel<WithHeader<Serv, Kvp>>
    where
        V: Into<Kvp::Value>,
    {
        self.inner.value(value).layer(self.service)
    }

    #[inline]
    pub fn parse_value<S>(
        self,
        value: S,
    ) -> Result<AuthChannel<WithHeader<Serv, Kvp>>, <Kvp::Value as FromStr>::Err>
    where
        S: AsRef<str>,
    {
        let layer = self.inner.parse_value(value.as_ref())?;
        Ok(layer.layer(self.service))
    }

    #[inline]
    pub fn try_value<V>(self, value: V) -> Result<AuthChannel<WithHeader<Serv, Kvp>>, V::Error>
    where
        V: TryInto<Kvp::Value>,
    {
        let layer = self.inner.try_value(value)?;
        Ok(layer.layer(self.service))
    }
}

impl<Kvp, Serv> From<AuthChannel<Serv>> for WithHeaderBuilder<Serv, Kvp, (), ()>
where
    Kvp: KeyValuePair,
{
    fn from(service: AuthChannel<Serv>) -> Self {
        Self::from_service(service)
    }
}
