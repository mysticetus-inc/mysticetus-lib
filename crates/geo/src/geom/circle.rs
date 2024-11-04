use std::f64::consts;

use super::nested::{Line, Polygon};
use crate::{InvalidCoordinate, NormalVec, Point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    center: NormalVec,
    radius_m: f64,
}

impl Circle {
    pub fn new<P>(center: P, radius_m: f64) -> Self
    where
        P: Into<NormalVec>,
    {
        Self {
            center: center.into(),
            radius_m,
        }
    }

    pub fn contains(&self, pt: Point) -> bool {
        let pt = NormalVec::from(pt);
        self.radius_m > pt.distance_to_m(&self.center)
    }

    /*
    pub fn geohash(&self) -> Result<crate::hash::GeoHash, crate::InvalidCoordinate> {
        let pt: Point = self.center.try_into()?;

        let on_side = self.center.extend_point(0.0, self.radius_m);
        let radius_angle =
            crate::ang::Degrees::new(self.center.angle_between(&on_side).to_degrees());

        let scale = crate::hash::Scale::determine_ideal_scale(radius_angle).unwrap();

        Ok(crate::hash::GeoHash::new_scale(pt, scale))
    }
    */

    pub fn to_region(&self) -> Result<crate::region::Region, crate::InvalidCoordinate> {
        let left = self.center.extend_point(0.0, self.radius_m).try_into()?;
        let top = self
            .center
            .extend_point(consts::FRAC_PI_2, self.radius_m)
            .try_into()?;
        let right = self
            .center
            .extend_point(consts::PI, self.radius_m)
            .try_into()?;
        let bottom = self
            .center
            .extend_point(consts::PI + consts::FRAC_PI_2, self.radius_m)
            .try_into()?;

        let mut region = crate::region::Region::from_point(left);
        region.add_point(top);
        region.add_point(right);
        region.add_point(bottom);
        Ok(region)
    }

    pub fn to_polygon(&self, output_points: usize) -> Result<Polygon, InvalidCoordinate> {
        let mut points = Vec::with_capacity(output_points);

        let interval = std::f64::consts::TAU / output_points as f64;

        let mut current_offset = 0.0;

        while current_offset <= std::f64::consts::TAU {
            let point = self.center.extend_point(current_offset, self.radius_m);

            let geojson_point = Point::try_from(point)?;

            points.push(geojson_point);

            current_offset += interval;
        }

        Ok(Polygon::from(vec![Line::from(points)]))
    }
}
