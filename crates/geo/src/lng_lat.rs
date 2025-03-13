//! [`Latitude`] and [`Longitude`] definitions

use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::FpCategory;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;

use serde::{Deserialize, Serialize, de, ser};

use crate::ang::Degrees;

const MINIMUM_LEGAL_VALUE: i64 = i64::MIN + 1;

macro_rules! impl_lat_lon {
    ($(($name:ident: $min:expr => $max:expr)),* $(,)?) => {
        $(
            #[doc = concat!(
                " A valid ",
                stringify!(name),
                " on the Earth, in degrees.\n\n",
                " Thin wrapper around a scaled [`i64`], which validates that the valid is not\n",
                " [`f64::NAN`], [`f64::INFINITY`], [`f64::NEG_INFINITY`], or outside of the\n",
                " range of valid values for this coordinate type ([",
                stringify!($min),
                ", ",
                stringify!($max),
                "])",
            )]
            #[repr(transparent)]
            #[derive(Default, Clone, Copy)]
            pub struct $name(i64);

            #[cfg(feature = "deepsize")]
            deepsize::known_deep_size!(0; $name);

            impl $name {
                #[doc = concat!(" The minimum valid value for [`", stringify!($name), "`]")]
                pub const MIN: Self = unsafe { Self::new_unchecked($min) };

                #[doc = concat!(" The zero value for [`", stringify!($name), "`]")]
                pub const ZERO: Self = Self(0);

                #[doc = concat!(" The maximum valid value for [`", stringify!($name), "`]")]
                pub const MAX: Self = unsafe { Self::new_unchecked($max) };

                #[doc = concat!(" The minimum valid value for [`", stringify!($name), "`], as an [`f64`]")]
                pub const MIN_F64: f64 = $min;

                #[doc = concat!(" The maximum valid value for [`", stringify!($name), "`] as an [`f64`]")]
                pub const MAX_F64: f64 = $max;


                /// Helper constant to scale the inner integer to a proper float.
                const SCALE_BY: f64 = MINIMUM_LEGAL_VALUE as f64 / $min;

                #[doc = concat!(
                    " Creates a new [`",
                    stringify!($name),
                    "`], validating that the floating point value is not NaN, ",
                    "+/- Infinity, or outside of the range [[`MIN`], [`MAX`]].",
                    "\n\n[`MIN`]: [`Self::MIN`]\n[`MAX`]: [`Self::MAX`]",
                )]
                #[inline]
                pub const fn new_checked(value: f64) -> Result<Self, InvalidCoordinate> {
                    match value.classify() {
                        FpCategory::Nan => {
                            Err(InvalidCoordinate::new(
                                CoordinateType::$name,
                                InvalidCoordinateReason::IsNaN,
                            ))
                        },
                        FpCategory::Infinite => {
                            Err(InvalidCoordinate::new(
                                CoordinateType::$name,
                                InvalidCoordinateReason::IsInf,
                            ))
                        },
                        _ if Self::MAX_F64 < value => {
                            Err(InvalidCoordinate::new(
                                CoordinateType::$name,
                                InvalidCoordinateReason::AboveMaximum {
                                    max: Self::MAX_F64,
                                    value,
                                }
                            ))
                        },
                        _ if Self::MIN_F64 > value => {
                            Err(InvalidCoordinate::new(
                                CoordinateType::$name,
                                InvalidCoordinateReason::BelowMinimum {
                                    min: Self::MIN_F64,
                                    value,
                                }
                            ))
                        },
                        _ => Ok(unsafe { Self::new_unchecked(value) }),
                    }
                }

                /// Identical to [`new_checked`], but panics on [`Err`].
                ///
                /// It replicates the body of [`new_checked`], but panics on the error conditions.
                ///
                /// TODO: Once panic message formatting is const-stable, refactor just use a match
                /// on the result of [`new_checked`].
                ///
                /// [`new_checked`]: [`Self::new_checked`]
                #[inline]
                pub const fn new(value: f64) -> Self {
                    match value.classify() {
                        FpCategory::Nan => panic!("value cannot be 'NaN'"),
                        FpCategory::Infinite => panic!("value cannot be +/- Inf"),
                        _ if Self::MAX_F64 < value => {
                            panic!("value is higher than the maximum valid value")
                        },
                        _ if Self::MIN_F64 > value => {
                            panic!("value is below the minimum valid value")
                        },
                        _ => unsafe { Self::new_unchecked(value) },
                    }
                }


                #[doc = concat!(
                    " Assembles this [`",
                    stringify!($name),
                    "`] with no checks at all on the input [`f64`]",
                )]
                #[inline]
                pub const unsafe fn new_unchecked(value: f64) -> Self {
                    Self((value * Self::SCALE_BY) as i64)
                }


                #[inline]
                pub const fn into_degrees(self) -> Degrees {
                    Degrees::new(self.get())
                }

                #[inline]
                pub fn delta(self, other: Self) -> Degrees {
                    self - other
                }

                #[inline]
                pub fn middle(self, other: Self) -> Self {
                    let half_delta = self.delta(other) / 2;

                    if self < other {
                        self + half_delta
                    } else {
                        other + half_delta
                    }
                }

                #[doc = concat!("Returns the underlying ", stringify!($name), " as a [`f64`]")]
                #[inline]
                pub const fn get(self) -> f64 {
                    self.0 as f64 / Self::SCALE_BY
                }
            }


            impl AddAssign<Degrees> for $name {
                #[inline]
                fn add_assign(&mut self, rhs: Degrees) {
                    *self = self.add(rhs);
                }
            }

            impl Sub<Degrees> for $name {
                type Output = Self;

                #[inline]
                fn sub(self, rhs: Degrees) -> Self::Output {
                    self.add(-rhs)
                }
            }

            impl SubAssign<Degrees> for $name {
                #[inline]
                fn sub_assign(&mut self, rhs: Degrees) {
                    *self = self.sub(rhs);
                }
            }



            impl fmt::Debug for $name {
                fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.debug_tuple(stringify!($name))
                        .field(&self.get())
                        .finish()
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "{}", self.get())
                }
            }

            impl FromStr for $name {
                type Err = InvalidCoordinate;

                #[inline]
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    s.trim()
                        .parse::<f64>()
                        .map_err(|err| {
                            let reason = InvalidCoordinateReason::ParseErr(err);
                            InvalidCoordinate::new(CoordinateType::$name, reason)
                        })
                        .and_then(|float| $name::new_checked(float))
                }
            }


            impl Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ser::Serializer
                {
                    serializer.serialize_f64(self.get())
                }
            }

            impl From<$name> for f64 {
                #[inline]
                fn from(c: $name) -> f64 {
                    c.get()
                }
            }

            impl PartialEq for $name {
                fn eq(&self, rhs: &Self) -> bool {
                    self.0 == rhs.0
                }
            }

            impl Eq for $name {}

            impl Hash for $name {
                #[inline]
                fn hash<H>(&self, hasher: &mut H)
                where
                    H: Hasher
                {
                    self.0.hash(hasher)
                }
            }

            impl PartialEq<f64> for $name {
                fn eq(&self, rhs: &f64) -> bool {
                    match Self::new_checked(*rhs) {
                        Ok(parsed) => *self == parsed,
                        Err(_) => false,
                    }
                }
            }

            impl PartialOrd for $name {
                fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(rhs))
                }
            }

            impl PartialOrd<f64> for $name {
                fn partial_cmp(&self, rhs: &f64) -> Option<std::cmp::Ordering> {
                    match Self::new_checked(*rhs) {
                        Ok(rhs) => Some(self.cmp(&rhs)),
                        Err(_) => None,
                    }
                }
            }

            impl Ord for $name {
                fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
                    self.0.cmp(&rhs.0)
                }
            }

            impl CoordinateAxis for $name {
                const MIN: Self = Self::MIN;
                const MAX: Self = Self::MAX;

                fn middle(self, other: Self) -> Self {
                    self.middle(other)
                }
            }


            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: de::Deserializer<'de>
                {

                    struct Visitor;

                    impl<'de> de::Visitor<'de> for Visitor {
                        type Value = $name;

                        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                            write!(f, "a valid {}, in degrees", CoordinateType::$name.as_str())
                        }

                        fn visit_u8<E>(self, u: u8) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_i8<E>(self, u: i8) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_u16<E>(self, u: u16) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_i16<E>(self, u: i16) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_u32<E>(self, u: u32) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_i32<E>(self, u: i32) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_u64<E>(self, u: u64) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_i64<E>(self, u: i64) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_u128<E>(self, u: u128) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_i128<E>(self, u: i128) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            self.visit_f64(u as f64)
                        }

                        fn visit_f32<E>(self, float: f32) -> Result<Self::Value, E>
                        where
                            E: de::Error
                        {
                            self.visit_f64(float as f64)
                        }

                        fn visit_f64<E>(self, float: f64) -> Result<Self::Value, E>
                        where
                            E: de::Error
                        {
                            $name::new_checked(float).map_err(|err| {
                                de::Error::invalid_value(de::Unexpected::Float(float), &err)
                            })
                        }

                        fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error
                        {
                            string.parse::<Self::Value>().map_err(|err| {
                                de::Error::invalid_value(de::Unexpected::Str(string), &err)
                            })
                        }

                        fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
                        where
                            E: de::Error
                        {
                            std::str::from_utf8(bytes)
                                .map_err(|_| {
                                    de::Error::invalid_value(de::Unexpected::Bytes(bytes), &self)
                                })
                                .and_then(|string| self.visit_str(string))
                        }
                    }


                    deserializer.deserialize_f64(Visitor)
                }
            }

            /// rand impl to generate random points for testing.
            #[cfg(any(test, feature = "random-geom"))]
            impl rand::distr::Distribution<$name> for rand::distr::StandardUniform {
                fn sample<R>(&self, rng: &mut R) -> $name
                where
                    R: rand::Rng + ?Sized
                {
                    $name(rng.random_range(MINIMUM_LEGAL_VALUE..i64::MAX))
                }
            }

            /// random helper functions for generating these for other types
            #[cfg(any(test, feature = "random-geom"))]
            impl $name {
                #[doc = concat!(
                    " Identical to calling [`rand::random::<",
                    stringify!($name),
                    ">`]`. Use [`",
                    stringify!($name),
                    "::random_from`] to generate a type from an existing [`rand::Rng`]"
                )]
                pub fn random() -> Self {
                    rand::random()
                }

                #[doc = concat!(
                    " Identical to calling [`rand::Rng::gen<",
                    stringify!($name),
                    ">`] on an existing source of [`rand::Rng`]."
                )]
                pub fn random_from<R>(rng: &mut R) -> Self
                where
                    R: rand::Rng + ?Sized
                {
                    rng.random()
                }
            }
        )*
    };
}

pub trait CoordinateAxis: Copy + Sub<Output = Degrees> {
    const MIN: Self;
    const MAX: Self;

    fn middle(self, other: Self) -> Self;
}

impl_lat_lon! {
    (Latitude: -90.0 => 90.0),
    (Longitude: -180.0 => 180.0),
}

// manual math impls to account for the different behavior for lat/lon
// In a nutshell:
// latitudes saturate at +/-90, since it doesn't wrap around.
// longitudes on the other hand do wrap around the anti-meridian.

impl Add<Degrees> for Latitude {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Degrees) -> Self::Output {
        let rhs_scaled = (rhs.get() * Self::SCALE_BY) as i64;
        Self(self.0.saturating_add(rhs_scaled).max(MINIMUM_LEGAL_VALUE))
    }
}

impl Sub for Latitude {
    type Output = Degrees;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let abs_diff = self.0.abs_diff(rhs.0) as f64 / Self::SCALE_BY;

        if self.0 > rhs.0 {
            Degrees::new(abs_diff)
        } else {
            Degrees::new(-abs_diff)
        }
    }
}

// longitude math:

impl Add<Degrees> for Longitude {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Degrees) -> Self::Output {
        let rhs_scaled = (rhs.get() * Self::SCALE_BY) as i64;
        Self(self.0.wrapping_add(rhs_scaled).max(MINIMUM_LEGAL_VALUE))
    }
}

impl Sub for Longitude {
    type Output = Degrees;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        const HALF_RANGE: u64 = u64::MAX / 2;

        let diff = self.0.abs_diff(rhs.0);

        let shortest = if diff < HALF_RANGE {
            diff
        } else {
            u64::MAX - diff
        };

        Degrees::new(shortest as f64 / Self::SCALE_BY)
    }
}

/// Enum containing the valid coordinate types. Used to generalize the error returned from
/// [`Latitude::new_checked`] and [`Longitude::new_checked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "aide", derive(schemars::JsonSchema))]
pub enum CoordinateType {
    Altitude,
    Latitude,
    Longitude,
}

impl CoordinateType {
    /// Returns the name of the variant as an lowercase `&'static str`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Altitude => "altitude",
            Self::Latitude => "latitude",
            Self::Longitude => "longitude",
        }
    }
}

/// An error returned by [`Latitude::new_checked`] and [`Longitude::new_checked`], if the value
/// passed in was either [`f64::NAN`], [`f64::INFINITY`], [`f64::NEG_INFINITY`], or out of range
/// for that unit.
///
/// [`Self::coordinate`] is the name of the coordinate for printing in error messages, and
/// [`Self::reason`] is what caused the error.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "aide", derive(schemars::JsonSchema))]
pub struct InvalidCoordinate {
    coordinate: CoordinateType,
    reason: InvalidCoordinateReason,
}

impl InvalidCoordinate {
    /// Internal builder for InvalidCoordinate errors.
    pub(crate) const fn new(coordinate: CoordinateType, reason: InvalidCoordinateReason) -> Self {
        Self { coordinate, reason }
    }

    /// Returns the coordinate type that caused the error.
    pub const fn coordinate(&self) -> CoordinateType {
        self.coordinate
    }

    /// Returns the reason the coordinate is invalid.
    pub const fn reason(&self) -> &InvalidCoordinateReason {
        &self.reason
    }
}

/// The reason why a coordinate failed to build/perform an operation successfully.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "aide", derive(schemars::JsonSchema))]
pub enum InvalidCoordinateReason {
    IsNaN,
    IsInf,
    #[serde(serialize_with = "serialize_display")]
    #[cfg_attr(feature = "aide", schemars(with = "String"))]
    ParseErr(std::num::ParseFloatError),
    BelowMinimum {
        min: f64,
        value: f64,
    },
    AboveMaximum {
        max: f64,
        value: f64,
    },
}

pub fn serialize_display<T, S>(disp: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: fmt::Display,
    S: serde::Serializer,
{
    serializer.serialize_str(&disp.to_string())
}

impl fmt::Display for InvalidCoordinateReason {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IsNaN => write!(formatter, "'NaN'"),
            Self::IsInf => write!(formatter, "infinite"),
            Self::ParseErr(err) => write!(formatter, "{err}"),
            Self::BelowMinimum { min, value } => {
                write!(
                    formatter,
                    "below the minimum valid value {min}. recieved value {value})"
                )
            }
            Self::AboveMaximum { max, value } => {
                write!(
                    formatter,
                    "above the maximum valid value {max}. recieved value {value})"
                )
            }
        }
    }
}

/*
#[cfg(feature = "aide")]
impl schemars::JsonSchema for Latitude {
    fn schema_name() -> String {
        format!("Latitude")
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::Number,
            ))),
            number: Some(Box::new(schemars::schema::NumberValidation {
                minimum: Some(Self::MIN.get()),
                maximum: Some(Self::MAX.get()),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

#[cfg(feature = "aide")]
impl schemars::JsonSchema for Longitude {
    fn schema_name() -> String {
        format!("Longitude")
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::Number,
            ))),
            number: Some(Box::new(schemars::schema::NumberValidation {
                minimum: Some(Self::MIN.get()),
                maximum: Some(Self::MAX.get()),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
*/

impl std::error::Error for InvalidCoordinate {}

impl fmt::Display for InvalidCoordinate {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} invalid: cannot be {}",
            self.coordinate.as_str(),
            self.reason
        )
    }
}

impl de::Expected for InvalidCoordinate {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{self}")
    }
}

#[cfg(test)]
mod tests {
    use super::{Degrees, Latitude, Longitude};
    const THRESH: f64 = 1e-9;

    macro_rules! assert_almost_eq {
        ($a:expr, $b:literal $(,)?) => {{
            let abs = ($a.get() - $b).abs();
            if !(abs <= THRESH) {
                panic!(
                    concat!(
                        "(",
                        stringify!($a),
                        " - ",
                        stringify!($b),
                        ").abs() > 1e-9 (lhs = {})"
                    ),
                    abs
                );
            }
        }};
        ($a:expr, $b:expr $(,)?) => {{
            let abs = ($a.get() - $b.get()).abs();
            if !(abs <= THRESH) {
                panic!(
                    concat!(
                        "(",
                        stringify!($a),
                        " - ",
                        stringify!($b),
                        ").abs() > 1e-9 (lhs = {})"
                    ),
                    abs
                );
            }
        }};
    }

    #[test]
    fn lng_lat_test_basic() {
        assert_almost_eq!(Longitude::MAX, 180.0);
        assert_almost_eq!(Latitude::MAX, 90.0);

        assert_almost_eq!(Latitude::MAX - Latitude::MAX, Latitude::ZERO);
        assert_almost_eq!(Longitude::MAX - Longitude::MAX, Longitude::ZERO);

        let middle_lat = Latitude::MAX.middle(Latitude::MIN);
        let middle_lon = Longitude::MAX.middle(Longitude::ZERO);

        let delta = Longitude::MAX - Longitude::ZERO;
        assert_almost_eq!(delta, 180.0);

        assert_almost_eq!(Latitude::ZERO, middle_lat);
        assert_almost_eq!(Longitude::new(Longitude::MAX_F64 / 2.0), middle_lon);

        let middle_high_lat = Latitude::MAX.middle(Latitude::ZERO);
        let middle_high_lon = Longitude::MAX.middle(Longitude::ZERO);

        assert_almost_eq!(middle_high_lat, 45.0);
        assert_almost_eq!(middle_high_lon, 90.0);

        assert_almost_eq!(Latitude::new(Latitude::MAX_F64 / 2.0), middle_high_lat);
        assert_almost_eq!(Longitude::new(Longitude::MAX_F64 / 2.0), middle_high_lon);

        let start = Longitude::new(179.0);
        let end = start + Degrees::new(2.0);

        assert_almost_eq!(end - start, start - end);
    }

    #[test]
    fn lng_lat_test_latitude_saturating() {
        assert_almost_eq!(Latitude::MAX, Latitude::MAX + Degrees::new(1.0));

        assert_almost_eq!(Latitude::MAX, Latitude::ZERO + Degrees::new(91.0));

        assert_almost_eq!(Latitude::MAX - Latitude::MAX, Degrees::ZERO);
    }

    #[test]
    fn lng_lat_test_longitude_wrapping() {
        let below_dateline = Longitude::MAX - Degrees::new(1.0);
        let above_dateline = Longitude::MIN + Degrees::new(1.0);

        let delta = below_dateline - above_dateline;

        assert_almost_eq!(delta, 2.0);
    }
}
