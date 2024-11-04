//! Retrieve the Mapbox tiles for a given map.
//!
//! For reference, the mapbox static tiles URL is in the form:
//! ```markdown
//! https://api.mapbox.com/styles/v1/{username}/{style_id}/tiles/{tilesize}/{z}/{x}/{y}?access_token={token}
//! ```

use crate::MapGeometry;
use crate::coords::Offset;

/// A specific tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tile {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

impl Tile {
    pub fn pixel_coords(self, geom: &MapGeometry) -> Offset {
        let start = Offset {
            x: (self.x - geom.tile.region.x_min) as i32 * crate::TILE_SIZE as i32,
            y: (self.y - geom.tile.region.y_min) as i32 * crate::TILE_SIZE as i32,
        };

        start - geom.pixel.center_offset
    }

    // for benchmarking
    #[cfg(feature = "benchmarking")]
    pub fn random<R: rand::Rng>(rng: &mut R, max_tile: u32) -> Self {
        let x = rng.gen_range(0..=max_tile);
        let y = rng.gen_range(0..=max_tile);
        Self { x, y }
    }
}
