//! Helper macros for dealing with GeoJson deserialization.

/// Helper macro for building deserialization errors.
macro_rules! de_err {
    ($fn_name:ident($($args:tt)+)) => {{
        ::serde::de::Error::$fn_name($($args)+)
    }};
    (type $variant:ident($unexpected:expr), $($args:tt)+) => {{
        ::serde::de::Error::invalid_type(
            ::serde::de::Unxpected::$variant($unexpected),
            $($args)+
        )
    }};
    (value $variant:ident($unexpected:expr): $($args:expr),+) => {{
        struct Expecting;

        impl ::serde::de::Expected for Expecting {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(formatter, "one of: ")?;
                $(
                    write!(formatter, "{}, ", $args)?;
                )+
                Ok(())
            }
        }

        ::serde::de::Error::invalid_value(
            ::serde::de::Unexpected::$variant($unexpected),
            &Expecting,
        )
    }};
    (value $variant:ident($unexpected:expr): $arg:expr) => {{
        struct Expecting;

        impl ::serde::de::Expected for Expecting {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(formatter, ": {}", $arg)
            }
        }

        ::serde::de::Error::invalid_value(
            ::serde::de::Unexpected::$variant($unexpected),
            &Expecting,
        )
    }};
    (missing $string:literal) => {{
        ::serde::de::Error::missing_field($string)
    }}
}

pub(crate) use de_err;
