//! A generalized [`Ellipse`] definition.

use super::nested::{Line, Polygon};
use crate::{InvalidCoordinate, NormalVec, Point};

pub struct Ellipse {
    center: NormalVec,
    semi_major_m: f64,
    semi_minor_m: f64,
    semi_major_bearing_rad: f64,
}

impl Ellipse {
    pub fn new<P>(
        center: P,
        semi_major_m: f64,
        semi_minor_m: f64,
        semi_major_bearing_rad: f64,
    ) -> Self
    where
        P: Into<NormalVec>,
    {
        Self {
            center: center.into(),
            semi_minor_m,
            semi_major_m,
            semi_major_bearing_rad,
        }
    }

    pub fn to_polygon(&self, output_points: usize) -> Result<Polygon, InvalidCoordinate> {
        let mut points = Vec::with_capacity(output_points);

        let interval = std::f64::consts::TAU / output_points as f64;

        let mut current_offset = 0.0;

        let major_sq = self.semi_major_m.powi(2);
        let minor_sq = self.semi_minor_m.powi(2);

        let numerator = self.semi_major_m * self.semi_minor_m;

        while current_offset <= std::f64::consts::TAU {
            let (sin_angle, cos_angle) = (self.semi_major_bearing_rad - current_offset).sin_cos();

            let major_term = major_sq * sin_angle * sin_angle;
            let minor_term = minor_sq * cos_angle * cos_angle;

            let distance_m = numerator / (major_term + minor_term).sqrt();

            let point = self.center.extend_point(current_offset, distance_m);

            let geojson_point = Point::try_from(point)?;

            points.push(geojson_point);

            current_offset += interval;
        }

        Ok(Polygon::from(vec![Line::from(points)]))
    }
}

#[test]
fn test_ellipse() {
    use plotters::prelude::*;
    let area = BitMapBackend::new("test.png", (1028, 1028)).into_drawing_area();

    area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&area)
        .build_cartesian_2d(-1.0f64..1.0f64, -1.0f64..1.0f64)
        .unwrap();

    let point = Point::new_checked(0.0, 0.0).unwrap();

    let ellipse = Ellipse {
        center: point.into(),
        semi_major_m: 90000.0,
        semi_minor_m: 30000.0,
        semi_major_bearing_rad: 45.0f64.to_radians(),
    };

    let poly = ellipse.to_polygon(300).unwrap();

    let point_iter = poly.into_iter().flatten().map(|x| {
        let (x, y) = x.as_lon_lat();
        println!("{} {}", x, y);
        Circle::new((x, y), 2, RED.filled())
    });

    chart.draw_series(point_iter).unwrap();
}

#[test]
fn test_ellipse_to_poly() -> Result<(), InvalidCoordinate> {
    let point = Point::new_checked(-121.22764556811171, 44.12679669837136)?;

    let ellipse = Ellipse {
        center: point.into(),
        semi_major_m: 1500.0,
        semi_minor_m: 1000.0,
        semi_major_bearing_rad: 45.0f64.to_radians(),
    };

    let poly = ellipse.to_polygon(10)?;

    Ok(())
}
