use std::fmt;

use nalgebra::Vector3;

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct NormalVec(Vector3<f64>);

impl fmt::Debug for NormalVec {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("NormalVec")
            .field("x", &self.0[0])
            .field("y", &self.0[1])
            .field("z", &self.0[2])
            .finish()
    }
}

impl NormalVec {
    /// A [`NormalVec`] pointing at the North Pole. Since Great Circles are represented by a
    /// normal vector, this also represents the Equator.
    pub const NORTH_POLE: Self = Self(vector![0.0, 0.0, 1.0]);

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(vector![x, y, z].normalize())
    }

    #[inline]
    pub fn from_point(point: super::Point) -> Self {
        let (sin_lat, cos_lat) = point.latitude_f64().to_radians().sin_cos();
        let (sin_lon, cos_lon) = point.longitude_f64().to_radians().sin_cos();

        Self(vector![cos_lat * cos_lon, cos_lat * sin_lon, sin_lat])
    }

    #[inline]
    pub fn x(&self) -> f64 {
        self.0[0]
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.0[1]
    }

    #[inline]
    pub fn z(&self) -> f64 {
        self.0[2]
    }

    #[inline]
    pub fn x_mut(&mut self) -> &mut f64 {
        &mut self.0[0]
    }

    #[inline]
    pub fn y_mut(&mut self) -> &mut f64 {
        &mut self.0[1]
    }

    #[inline]
    pub fn z_mut(&mut self) -> &mut f64 {
        &mut self.0[2]
    }

    pub fn great_circle(&self, other: &Self) -> Self {
        Self(self.0.cross(&other.0))
    }

    pub fn angle_between(&self, other: &Self) -> f64 {
        let cross_mag = self.0.cross(&other.0).norm();
        let dot = self.0.dot(&other.0);

        cross_mag.atan2(dot)
    }

    pub fn abs_unit_dist(&self, other: &Self) -> f64 {
        (self.0 - other.0).magnitude().abs()
    }

    pub fn distance_to_m(&self, other: &Self) -> f64 {
        self.angle_between(other) * crate::R_EARTH_METERS
    }

    pub fn bearing_to_rad(&self, other: &Self) -> f64 {
        let c1 = self.great_circle(other);
        let c2 = self.great_circle(&Self::NORTH_POLE);

        let c1_x_c2 = c1.0.cross(&c2.0);

        let sin_angle = c1_x_c2.norm() * c1_x_c2.dot(&self.0).signum();
        let cos_angle = c1.0.dot(&c2.0);

        let angle = sin_angle.atan2(cos_angle).to_degrees();

        (angle + 180.0) % 360.0
    }

    pub fn midpoint(&self, other: &Self) -> Self {
        Self((self.0 + other.0).normalize())
    }

    pub fn intermediate_point(&self, other: &Self, frac_between: f64) -> Self {
        let delta = other.0 - self.0;

        let between = self.0 + frac_between * delta;

        Self(between.normalize())
    }

    pub fn extend_point(&self, bearing_rad: f64, distance_m: f64) -> Self {
        let east = Self::NORTH_POLE.0.cross(&self.0).normalize();
        let north = self.0.cross(&east);

        let (sin_bearing, cos_bearing) = bearing_rad.sin_cos();

        let direc = cos_bearing * north + sin_bearing * east;

        let central_angle = distance_m / crate::R_EARTH_METERS;
        let (sin_dist, cos_dist) = central_angle.sin_cos();

        let dst = cos_dist * self.0 + sin_dist * direc;

        Self(dst)
    }

    pub fn circle_around(
        self,
        radius_m: f64,
        points: usize,
    ) -> impl ExactSizeIterator<Item = Self> + Send + 'static {
        assert!(1 < points, "a generated circle must have more than 1 point");

        const TWO_PI: f64 = 2.0 * std::f64::consts::PI;

        let make_point_for_index = move |i| {
            let bearing_rad = TWO_PI * ((i as f64) / (points as f64));
            self.extend_point(bearing_rad, radius_m)
        };

        (0..points).map(make_point_for_index)
    }

    #[inline]
    pub fn circle_around_as_points(
        self,
        radius_m: f64,
        points: usize,
    ) -> impl ExactSizeIterator<Item = Result<crate::Point, crate::lng_lat::InvalidCoordinate>>
    + Send
    + 'static {
        self.circle_around(radius_m, points)
            .map(Self::try_into_point)
    }

    #[inline]
    pub fn circle_around_into_line(
        self,
        radius_m: f64,
        points: usize,
    ) -> Result<crate::geom::Line, crate::lng_lat::InvalidCoordinate> {
        let mut line = crate::geom::Line::with_capacity(points + 1);

        for result in self.circle_around_as_points(radius_m, points) {
            let point = result?;
            line.push(point);
        }

        if line.as_slice().first() != line.as_slice().last() {
            line.push(line.as_slice()[0]);
        }

        Ok(line)
    }

    #[inline]
    pub fn try_into_point(self) -> Result<crate::Point, crate::lng_lat::InvalidCoordinate> {
        self.try_into()
    }
}

impl From<crate::Point> for NormalVec {
    #[inline]
    fn from(point: crate::Point) -> Self {
        Self::from_point(point)
    }
}

impl TryFrom<NormalVec> for crate::Point {
    type Error = crate::lng_lat::InvalidCoordinate;

    #[inline]
    fn try_from(value: NormalVec) -> Result<Self, Self::Error> {
        let inner = value.0.normalize();

        let denom = (inner[0].powi(2) + inner[1].powi(2)).sqrt();

        let latitude = inner[2].atan2(denom).to_degrees();
        let longitude = inner[1].atan2(inner[0]).to_degrees();

        crate::Point::new_checked(longitude, latitude)
    }
}
