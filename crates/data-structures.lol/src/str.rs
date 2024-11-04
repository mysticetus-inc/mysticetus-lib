//! Wrappers around [`Box<str>`], [`Arc<str>`] and [`Rc<str>`] that provide more friendly trait
//! implementations for strings.
use std::borrow::{Borrow, BorrowMut, Cow};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

macro_rules! impl_str_wrapper {
    ($($name:ident($type:ident<str> $($const:tt)?)),* $(,)?) => {
        $(
            #[doc = concat!(
                " Wrapper around [`",
                stringify!($type),
                "<str>`]. Needed since [`",
                stringify!($type),
                "<str>`] doesn't implement several ",
            )]
            #[doc = " traits that are nice to have with strings, such as [`Borrow<str>`], "]
            #[doc = " which this type provides."]
            #[cfg_attr(feature = "deepsize", derive(deepsize::DeepSizeOf))]
            pub struct $name($type<str>);

            impl Default for $name {
                #[inline]
                fn default() -> Self {
                    Self($type::from(""))
                }
            }

            impl Hash for $name {
                #[inline]
                fn hash<H>(&self, hasher: &mut H)
                where
                    H: Hasher
                {
                    self.as_str().hash(hasher);
                }
            }

            impl Clone for $name {
                #[inline]
                fn clone(&self) -> Self {
                    Self($type::clone(&self.0))
                }
            }

            impl $name {
                #[inline]
                pub $($const)? fn as_str(&self) -> &str {
                    &*self.0
                }

                #[inline]
                pub fn as_inner(&self) -> &$type<str> {
                    &self.0
                }

                #[inline]
                pub fn as_inner_mut(&mut self) -> &mut $type<str> {
                    &mut self.0
                }


                #[inline]
                pub fn into_inner(self) -> $type<str> {
                    self.0
                }
            }

            impl fmt::Debug for $name {
                #[inline]
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.as_str().fmt(formatter)
                }
            }

            impl fmt::Display for $name {
                #[inline]
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.as_str().fmt(formatter)
                }
            }

            impl<T> PartialEq<T> for $name
            where
                T: AsRef<str> + ?Sized
            {
                #[inline]
                fn eq(&self, other: &T) -> bool {
                    self.as_str().eq(other.as_ref())
                }
            }

            impl Eq for $name { }

            impl<T> PartialOrd<T> for $name
            where
                T: AsRef<str> + ?Sized
            {
                #[inline]
                fn partial_cmp(&self, other: &T) -> Option<Ordering> {
                    Some(self.as_str().cmp(other.as_ref()))
                }
            }

            impl Ord for $name {
                #[inline]
                fn cmp(&self, other: &Self) -> Ordering {
                    self.as_str().cmp(other.as_str())
                }
            }

            impl AsRef<str> for $name {
                #[inline]
                fn as_ref(&self) -> &str {
                    self.as_str()
                }
            }

            impl AsRef<[u8]> for $name {
                #[inline]
                fn as_ref(&self) -> &[u8] {
                    self.as_str().as_bytes()
                }
            }

            impl Borrow<str> for $name {
                #[inline]
                fn borrow(&self) -> &str {
                    self.as_str()
                }
            }

            impl Deref for $name {
                type Target = str;

                #[inline]
                fn deref(&self) -> &Self::Target {
                    self.as_str()
                }
            }

            impl<T> From<T> for $name
            where
                $type<str>: From<T>
            {
                #[inline]
                fn from(s: T) -> Self {
                    Self(From::from(s))
                }
            }

            impl std::str::FromStr for $name {
                type Err = !;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Ok(Self::from(s))
                }
            }

            impl serde::Serialize for $name {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer
                {
                    serializer.serialize_str(self.as_str())
                }
            }

            impl<'de> serde::Deserialize<'de> for $name {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>
                {
                    match Cow::<'de, str>::deserialize(deserializer) {
                        Ok(s) => Ok($name::from(s)),
                        Err(err) => Err(err),
                    }
                }
            }
        )*
    };
}

impl_str_wrapper! {
    BoxStr(Box<str>),
    RcStr(Rc<str>),
    ArcStr(Arc<str>),
}

// since Arc<str> and Rc<str> cant be mutated, we need to add these
// mutable variants of the above traits for BoxStr.

impl AsMut<str> for BoxStr {
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl BorrowMut<str> for BoxStr {
    #[inline]
    fn borrow_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl DerefMut for BoxStr {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_str()
    }
}

impl BoxStr {
    #[inline]
    pub const fn as_mut_str(&mut self) -> &mut str {
        &mut self.0
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.0.into_string()
    }
}
