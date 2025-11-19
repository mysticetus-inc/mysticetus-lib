use std::fmt;
use std::hash::{Hash, Hasher};

use serde::ser::{self, SerializeSeq};
use serde::{Deserialize, Serialize, de};

use crate::ang::Degrees;
// use crate::hash::GeoHash;
use crate::lng_lat::{InvalidCoordinate, Latitude, Longitude};

/// Represents a valid point on the Earth
#[derive(Debug, Clone, Copy)]
pub struct Point {
    longitude: Longitude,
    latitude: Latitude,
    altitude: Option<f64>,
}

/*
#[cfg(feature = "aide")]
impl schemars::JsonSchema for Point {
    fn schema_name() -> String {
        format!("Point")
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::Array,
            ))),
            array: Some(Box::new(schemars::schema::ArrayValidation {
                items: Some(schemars::schema::SingleOrVec::Vec(vec![
                    gen.subschema_for::<Longitude>(),
                    gen.subschema_for::<Latitude>(),
                    gen.subschema_for::<Option<f64>>(),
                ])),
                min_items: Some(2),
                max_items: Some(3),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
*/

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.longitude == other.longitude && self.latitude == other.latitude
    }
}

impl Eq for Point {}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.longitude.hash(state);
        self.latitude.hash(state);
    }
}

impl Point {
    /// Assembles a [`Point`] from a [`Longitude`] and [`Latitude`].
    pub const fn new(longitude: Longitude, latitude: Latitude) -> Self {
        Self {
            longitude,
            latitude,
            altitude: None,
        }
    }

    /// Assembles a [`Point`] from a [`Longitude`], [`Latitude`] and 'altitude'.
    pub const fn new_with_alt(longitude: Longitude, latitude: Latitude, altitude: f64) -> Self {
        Self {
            longitude,
            latitude,
            altitude: Some(altitude),
        }
    }

    /// If the altitude is an invalid number (i.e not finite), remove it.
    pub fn fix_altitude(&mut self) {
        if let Some(alt) = self.altitude {
            if !alt.is_finite() {
                self.altitude = None;
            }
        }
    }

    #[inline]
    pub const fn new_raw(longitude: f64, latitude: f64) -> Self {
        let longitude = match Longitude::new_checked(longitude) {
            Ok(lon) => lon,
            Err(_err) => panic!("invalid longitude"),
        };

        let latitude = match Latitude::new_checked(latitude) {
            Ok(lat) => lat,
            Err(_err) => panic!("invalid latitude"),
        };

        Self::new(longitude, latitude)
    }

    /// Assembles a [`Point`] from a longitude and latitude as [`f64`]'s. If either is invalid,
    /// an [`Err`] is returned.
    pub const fn new_checked(longitude: f64, latitude: f64) -> Result<Self, InvalidCoordinate> {
        let longitude = match Longitude::new_checked(longitude) {
            Ok(lon) => lon,
            Err(err) => return Err(err),
        };

        let latitude = match Latitude::new_checked(latitude) {
            Ok(lat) => lat,
            Err(err) => return Err(err),
        };

        Ok(Self {
            longitude,
            latitude,
            altitude: None,
        })
    }

    /*
    #[inline]
    /// Convienience method that just calls [`GeoHash::new`]
    pub fn geohash(self) -> GeoHash {
        GeoHash::new(self)
    }
    */

    /// Assembles a [`Point`] from a longitude, latitude and altitude as [`f64`]'s. If the
    /// latitude or longitude is invalid, an [`Err`] is returned. The current implementation
    /// performs no checks on the altitude, but this may change in the future.
    pub const fn new_checked_with_alt(
        longitude: f64,
        latitude: f64,
        altitude: f64,
    ) -> Result<Self, InvalidCoordinate> {
        let longitude = match Longitude::new_checked(longitude) {
            Ok(lon) => lon,
            Err(err) => return Err(err),
        };

        let latitude = match Latitude::new_checked(latitude) {
            Ok(lat) => lat,
            Err(err) => return Err(err),
        };
        Ok(Self {
            longitude,
            latitude,
            altitude: Some(altitude),
        })
    }

    /// Returns the [`Longitude`].
    pub const fn longitude(&self) -> Longitude {
        self.longitude
    }

    /// Returns the inner [`Latitude`].
    pub const fn latitude(&self) -> Latitude {
        self.latitude
    }

    pub fn angular_distance(&self, other: &Self) -> Degrees {
        let lat1 = self.latitude_f64().to_radians();
        let lon1 = self.longitude_f64().to_radians();
        let lat2 = other.latitude_f64().to_radians();
        let lon2 = other.longitude_f64().to_radians();

        let delta_lat = lat2 - lat1;
        let delta_lon = lon2 - lon1;

        let delta_lat_sin = (delta_lat / 2.0).sin();
        let delta_lon_sin = (delta_lon / 2.0).sin();

        let a_term_1 = delta_lat_sin * delta_lat_sin;
        let a_term_2 = lat1.cos() * lat2.cos() * delta_lon_sin * delta_lon_sin;

        let a = a_term_1 + a_term_2;

        let c = 2.0 * f64::atan2(a.sqrt(), (1.0 - a).sqrt());

        Degrees::new(c.to_degrees())
    }

    /// Returns the longitude as an [`f64`].
    pub const fn longitude_f64(&self) -> f64 {
        self.longitude.get()
    }

    /// Returns the latitude as an [`f64`].
    pub const fn latitude_f64(&self) -> f64 {
        self.latitude.get()
    }

    /// If it is set, returns the altitude.
    pub const fn altitude(&self) -> Option<f64> {
        self.altitude
    }

    /// Returns the longitude and latitude as a pair.
    pub const fn as_lon_lat(&self) -> (f64, f64) {
        (self.longitude.get(), self.latitude.get())
    }

    /// Returns the longitude, latitute, and optional altitude as a triplet.
    pub const fn as_lon_lat_alt(&self) -> (f64, f64, Option<f64>) {
        (self.longitude.get(), self.latitude.get(), self.altitude)
    }

    #[inline]
    pub fn into_normal_vec(self) -> crate::NormalVec {
        self.into()
    }

    pub fn circle_around(
        self,
        radius_m: f64,
        points: usize,
    ) -> impl ExactSizeIterator<Item = crate::NormalVec> {
        self.into_normal_vec().circle_around(radius_m, points)
    }

    pub fn circle_around_as_points(
        self,
        radius_m: f64,
        points: usize,
    ) -> impl ExactSizeIterator<Item = Result<Self, InvalidCoordinate>> {
        self.circle_around(radius_m, points).map(Self::try_from)
    }

    pub fn circle_around_to_line(
        self,
        radius_m: f64,
        points: usize,
    ) -> Result<crate::geom::Line, InvalidCoordinate> {
        let mut line = crate::geom::Line::with_capacity(points + 1);

        for result in self.circle_around_as_points(radius_m, points) {
            let point = result?;
            line.push(point);
        }

        Ok(line)
    }
}

impl From<(Latitude, Longitude)> for Point {
    #[inline]
    fn from((lat, lon): (Latitude, Longitude)) -> Self {
        Self::new(lon, lat)
    }
}

impl From<(Longitude, Latitude)> for Point {
    #[inline]
    fn from((lon, lat): (Longitude, Latitude)) -> Self {
        Self::new(lon, lat)
    }
}

impl TryFrom<(f64, f64)> for Point {
    type Error = InvalidCoordinate;

    fn try_from(lon_lat: (f64, f64)) -> Result<Self, Self::Error> {
        Self::new_checked(lon_lat.0, lon_lat.1)
    }
}

impl TryFrom<(f64, f64, f64)> for Point {
    type Error = InvalidCoordinate;

    fn try_from(value: (f64, f64, f64)) -> Result<Self, Self::Error> {
        Self::new_checked_with_alt(value.0, value.1, value.2)
    }
}

impl TryFrom<(f64, f64, Option<f64>)> for Point {
    type Error = InvalidCoordinate;

    fn try_from(value: (f64, f64, Option<f64>)) -> Result<Self, Self::Error> {
        match value {
            (lon, lat, Some(alt)) => Self::new_checked_with_alt(lon, lat, alt),
            (lon, lat, _) => Self::new_checked(lon, lat),
        }
    }
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let len = if self.altitude.is_some() { 3 } else { 2 };

        let mut serialize_seq = serializer.serialize_seq(Some(len))?;

        serialize_seq.serialize_element(&self.longitude)?;
        serialize_seq.serialize_element(&self.latitude)?;

        if let Some(alt) = self.altitude.as_ref() {
            serialize_seq.serialize_element(alt)?;
        }

        serialize_seq.end()
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(PointVisitor)
    }
}

struct PointVisitor;

impl<'de> de::Visitor<'de> for PointVisitor {
    type Value = Point;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "2-3 element array with a longitude, latitude and optional altitude"
        )
    }

    fn visit_seq<S>(self, mut seq_access: S) -> Result<Self::Value, S::Error>
    where
        S: de::SeqAccess<'de>,
    {
        let longitude: Longitude = seq_access
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        let latitude: Latitude = seq_access
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        let altitude: Option<f64> = seq_access.next_element()?;
        Ok(Point {
            longitude,
            latitude,
            altitude,
        })
    }
}

#[cfg(any(test, feature = "random-geom"))]
mod point_rand_impls {
    use rand::Rng;
    use rand::distr::{Distribution, StandardUniform};

    use super::{Latitude, Longitude, Point};

    impl Distribution<Point> for StandardUniform {
        fn sample<R>(&self, rng: &mut R) -> Point
        where
            R: Rng + ?Sized,
        {
            let longitude = Longitude::random_from(rng);
            let latitude = Latitude::random_from(rng);

            Point {
                longitude,
                latitude,
                altitude: None,
            }
        }
    }

    impl Point {
        pub fn random() -> Self {
            rand::random()
        }

        pub fn random_from<R>(rng: &mut R) -> Self
        where
            R: rand::Rng,
        {
            rng.random()
        }
    }
}
