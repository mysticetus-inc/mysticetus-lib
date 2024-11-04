use crate::protobuf::value::Kind;
use crate::protobuf::{ListValue, NullValue, Struct, Value};

impl<K> From<K> for Value
where
    Kind: From<K>,
{
    #[inline]
    fn from(kind: K) -> Self {
        Value {
            kind: Some(Kind::from(kind)),
        }
    }
}

impl<K> From<Option<K>> for Kind
where
    Kind: From<K>,
{
    #[inline]
    fn from(k: Option<K>) -> Self {
        match k {
            Some(k) => Kind::from(k),
            None => Kind::NullValue(NullValue::NullValue as i32),
        }
    }
}

impl<K, V> FromIterator<(K, V)> for Struct
where
    String: From<K>,
    Kind: From<V>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let fields = iter
            .into_iter()
            .map(|(k, v)| (String::from(k), Value::from(Kind::from(v))))
            .collect::<std::collections::HashMap<String, Value>>();

        Struct { fields }
    }
}

impl<K, V> Extend<(K, V)> for Struct
where
    String: From<K>,
    Kind: From<V>,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        self.fields.extend(
            iter.into_iter()
                .map(|(k, v)| (String::from(k), Value::from(Kind::from(v)))),
        );
    }
}

impl<K> FromIterator<K> for ListValue
where
    Kind: From<K>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = K>,
    {
        let values = iter
            .into_iter()
            .map(|v| Value::from(Kind::from(v)))
            .collect::<Vec<Value>>();

        ListValue { values }
    }
}

impl<K> Extend<K> for ListValue
where
    Kind: From<K>,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = K>,
    {
        self.values
            .extend(iter.into_iter().map(|v| Value::from(Kind::from(v))));
    }
}

impl From<String> for Kind {
    #[inline]
    fn from(s: String) -> Self {
        Kind::StringValue(s)
    }
}

impl From<bool> for Kind {
    #[inline]
    fn from(b: bool) -> Self {
        Kind::BoolValue(b)
    }
}

macro_rules! impl_from_ints {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for Kind {
                #[inline]
                fn from(n: $t) -> Self {
                    Kind::NumberValue(n as f64)
                }
            }
        )*
    };
}

impl_from_ints!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
