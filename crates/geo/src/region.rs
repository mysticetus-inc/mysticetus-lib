use std::ops::RangeInclusive;

use crate::ang::Degrees;
use crate::{Latitude, Longitude, Point};

/// A region on the Earth, bounded by 2 latitude + 2 longitude lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Region {
    /// The minimum bounding latitude.
    min_lat: Latitude,
    /// The minimum bounding longitude.
    min_lon: Longitude,
    /// The maximum bounding latitude.
    max_lat: Latitude,
    /// The maximum bounding longitude.
    max_lon: Longitude,
}

macro_rules! impl_point_fns {
    ($($fn_name:ident($lon_field:ident, $lat_field:ident)),* $(,)?) => {
        $(
            #[inline]
            pub const fn $fn_name(&self) -> Point {
                Point::new(self.$lon_field, self.$lat_field)
            }
        )*
    };
}

impl Region {
    /// Creates a region from a single point. The resulting region will have 0 area.
    #[inline]
    pub const fn from_point(pt: Point) -> Self {
        let lat = pt.latitude();
        let lon = pt.longitude();

        Self {
            min_lon: lon,
            min_lat: lat,
            max_lon: lon,
            max_lat: lat,
        }
    }

    /// Identical to [`Region::from_point`], but non-const. There's a weird compiler bug thats
    /// causing calls to the const version to blow up in non-const contexts, so this gets around
    /// that.
    #[inline]
    pub fn from_point_non_const(pt: Point) -> Self {
        let lat = pt.latitude();
        let lon = pt.longitude();

        Self {
            min_lon: lon,
            min_lat: lat,
            max_lon: lon,
            max_lat: lat,
        }
    }

    pub fn delta_lat(&self) -> Degrees {
        self.max_lat - self.min_lat
    }

    pub fn diagonal(&self) -> Degrees {
        let max_pt = Point::new(self.max_lon, self.max_lat);
        let min_pt = Point::new(self.min_lon, self.min_lat);

        max_pt.angular_distance(&min_pt)
    }

    pub fn delta_lon(&self) -> Degrees {
        self.max_lon - self.min_lon
    }

    pub fn center(&self) -> Point {
        let lat = self.max_lat.middle(self.min_lat);
        let lon = self.max_lon.middle(self.min_lon);

        Point::new(lon, lat)
    }

    #[inline]
    pub const fn lat_range(&self) -> RangeInclusive<Latitude> {
        self.min_lat..=self.max_lat
    }

    #[inline]
    pub const fn lon_range(&self) -> RangeInclusive<Longitude> {
        self.min_lon..=self.max_lon
    }

    impl_point_fns! {
        bottom_left(min_lon, min_lat),
        top_right(max_lon, max_lat),
        top_left(min_lon, max_lat),
        bottom_right(max_lon, min_lat),
    }

    pub fn contains(&self, pt: Point) -> bool {
        self.lat_range().contains(&pt.latitude()) && self.lon_range().contains(&pt.longitude())
    }

    pub fn overlaps_with(&self, other: &Self) -> bool {
        self.contains(other.bottom_left())
            || self.contains(other.bottom_right())
            || self.contains(other.top_left())
            || self.contains(other.top_right())
    }

    #[inline]
    pub fn add_lon(&mut self, lon: Longitude) {
        self.min_lon = self.min_lon.min(lon);
        self.max_lon = self.max_lon.max(lon);
    }

    #[inline]
    pub fn add_lat(&mut self, lat: Latitude) {
        self.min_lat = self.min_lat.min(lat);
        self.max_lat = self.max_lat.max(lat);
    }

    #[inline]
    pub fn add_point(&mut self, pt: Point) {
        self.add_lat(pt.latitude());
        self.add_lon(pt.longitude());
    }

    #[inline]
    pub fn try_from_iter<I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = Point>,
        I::IntoIter: Iterator,
    {
        let mut iter = iter.into_iter();

        let mut new = iter.next().map(Region::from_point_non_const)?;
        new.add_points(iter);
        Some(new)
    }

    #[inline]
    pub fn merge(&mut self, other: Region) {
        self.min_lon = self.min_lon.min(other.min_lon);
        self.max_lon = self.max_lon.max(other.max_lon);
        self.min_lat = self.min_lat.min(other.min_lat);
        self.max_lat = self.max_lat.max(other.max_lat);
    }

    #[inline]
    pub fn add_points<I>(&mut self, pts: I)
    where
        I: IntoIterator<Item = Point>,
        I::IntoIter: Iterator,
    {
        for point in pts {
            self.add_point(point);
        }
    }

    /// Returns true if the region has 0 area, i.e the min + max bounding lines are equal.
    ///
    /// ```
    /// # use geo::region::Region;
    /// # use geo::Point;
    /// let point = Point::new_checked(123.0, 45.0).unwrap();
    ///
    /// let mut region = Region::from_point(point);
    /// assert!(region.is_zero_area());
    ///
    /// let new_point = Point::new_checked(125.0, 42.0).unwrap();
    /// region.add_point(new_point);
    /// assert!(!region.is_zero_area());
    /// ```
    #[inline]
    pub fn is_zero_area(&self) -> bool {
        self.min_lat == self.max_lat || self.min_lon == self.max_lon
    }
}

impl Extend<Point> for Region {
    fn extend<T: IntoIterator<Item = Point>>(&mut self, iter: T) {
        self.add_points(iter);
    }
}
