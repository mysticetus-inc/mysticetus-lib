#![feature(
    array_try_from_fn,
    portable_simd,
    const_try,
    const_option_ext,
    const_trait_impl
)]

#[cfg(feature = "normal")]
#[macro_use]
extern crate nalgebra;

pub mod ang;
pub mod extent;
pub mod geom;
// pub mod hash;
pub mod lng_lat;
pub mod math;
#[cfg(feature = "normal")]
pub mod normal;
pub mod point;
pub mod region;
pub mod util;
pub mod wkb;

pub use lng_lat::{InvalidCoordinate, Latitude, Longitude};
#[cfg(feature = "normal")]
pub use normal::NormalVec;
pub use point::Point;

pub const R_EARTH_METERS: f64 = 6_378_100.0;

/*
trait GeometryNewType: Copy {
    type Inner;

    fn get(self) -> Self::Inner;
}
*/
