use std::fmt;
use std::marker::PhantomData;

use serde::de;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DebugVisitor<V, Tgt: DebugTarget = Stdout> {
    inner: V,
    logged: bool,
    _target: PhantomData<Tgt>,
}

impl<V, Tgt: DebugTarget> DebugVisitor<V, Tgt> {
    pub const fn new(inner: V) -> Self {
        Self {
            inner,
            logged: false,
            _target: PhantomData,
        }
    }
}

impl<V> DebugVisitor<V> {
    pub const fn stdout(inner: V) -> Self {
        Self::new(inner)
    }
}

impl<V> DebugVisitor<V, Stderr> {
    pub const fn stderr(inner: V) -> Self {
        Self::new(inner)
    }
}

impl<V> DebugVisitor<V, Tracing> {
    pub const fn tracing(inner: V) -> Self {
        Self::new(inner)
    }
}

pub trait DebugTarget {
    fn log<V, E>(value: &V, error: &E)
    where
        V: fmt::Debug + ?Sized,
        E: std::error::Error + ?Sized;
}

pub enum Stdout {}
pub enum Stderr {}
pub enum Tracing {}

impl DebugTarget for Stdout {
    fn log<V, E>(value: &V, error: &E)
    where
        V: fmt::Debug + ?Sized,
        E: std::error::Error + ?Sized,
    {
        println!("{error}: {value:?}")
    }
}

impl DebugTarget for Stderr {
    fn log<V, E>(value: &V, error: &E)
    where
        V: fmt::Debug + ?Sized,
        E: std::error::Error + ?Sized,
    {
        eprintln!("{error}: {value:?}")
    }
}

impl DebugTarget for Tracing {
    fn log<V, E>(value: &V, error: &E)
    where
        V: fmt::Debug + ?Sized,
        E: std::error::Error + ?Sized,
    {
        tracing::error!(message = "debug serde error", ?error, ?value);
    }
}

impl<V, Tgt: DebugTarget> From<V> for DebugVisitor<V, Tgt> {
    fn from(inner: V) -> Self {
        Self::new(inner)
    }
}

impl<'de, V, Tgt> de::DeserializeSeed<'de> for DebugVisitor<V, Tgt>
where
    V: de::DeserializeSeed<'de>,
    Tgt: DebugTarget,
{
    type Value = V::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.inner.deserialize(deserializer)
    }
}

macro_rules! visit_copy_type {
    ($(
        $fn_name:ident($arg_ty:ty)
    ),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<E>(self, v: $arg_ty) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match self.inner.$fn_name(v) {
                    Ok(value) => Ok(value),
                    Err(error) if self.logged => Err(error),
                    Err(error) => {
                        Tgt::log(&v, &error);
                        Err(error)
                    }
                }
            }
        )*
    };
}

impl<'de, V, Tgt> de::Visitor<'de> for DebugVisitor<V, Tgt>
where
    V: de::Visitor<'de>,
    Tgt: DebugTarget,
{
    type Value = V::Value;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.expecting(formatter)
    }

    visit_copy_type! {
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_u128(u128),
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_i128(i128),
        visit_f32(f32),
        visit_f64(f64),
        visit_char(char),
        visit_bool(bool),
        visit_str(&str),
        visit_borrowed_str(&'de str),
        visit_bytes(&[u8]),
        visit_borrowed_bytes(&'de [u8]),
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        struct DebugNoneTypeName<T>(PhantomData<T>);

        impl<T> fmt::Debug for DebugNoneTypeName<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Option::<{}>::None", std::any::type_name::<T>())
            }
        }

        match self.inner.visit_none() {
            Ok(value) => Ok(value),
            Err(error) if self.logged => Err(error),
            Err(error) => {
                Tgt::log(&DebugNoneTypeName(PhantomData::<V::Value>), &error);
                Err(error)
            }
        }
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        struct DebugUnitTypeName<T>(PhantomData<T>);

        impl<T> fmt::Debug for DebugUnitTypeName<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "(() as '{}')", std::any::type_name::<T>())
            }
        }

        match self.inner.visit_unit() {
            Ok(value) => Ok(value),
            Err(error) if self.logged => Err(error),
            Err(error) => {
                Tgt::log(&DebugUnitTypeName(PhantomData::<V::Value>), &error);
                Err(error)
            }
        }
    }

    // TODO: make a wrapper deserializer to properly handle this
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.inner.visit_newtype_struct(deserializer)
    }

    // TODO: make a wrapper map access to properly handle this
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.inner.visit_map(map)
    }

    // TODO: make a wrapper seq access to properly handle this
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.inner.visit_seq(seq)
    }

    // TODO: make a wrapper enum access to properly handle this
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.inner.visit_enum(data)
    }

    // TODO: make a wrapper deserializer to properly handle this
    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.inner.visit_some(deserializer)
    }
}
