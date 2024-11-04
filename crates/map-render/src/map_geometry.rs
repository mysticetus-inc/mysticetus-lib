use geo::Point;
use geo::region::Region;

use crate::coords::{MapPoint, Offset, Size, TileRegion, Zoom};

/// The finalized map geometry. The final, resulting map image is generated from this.
///
/// Split up into 3 fields to keep units/related fields grouped together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapGeometry {
    pub pixel: PixelExtent,
    pub map: MapExtent,
    pub tile: TileExtent,
}

/// Pixel Geometry of the map to be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelExtent {
    /// The size of the final, resulting image.
    pub image_size: Size<u32>,
    /// The size of the final image, if the entirety of all tiles were rendered
    /// (as the final image crops corners)
    pub tile_span: Size<u32>,
    pub center_offset: Offset,
}

impl PixelExtent {
    pub fn new(image_size: Size<u32>, tile_region: &TileRegion) -> Self {
        macro_rules! compute_offset {
            ($tile_span:expr, $img_size:expr) => {{ (($tile_span * crate::TILE_SIZE) as i32 - $img_size as i32) / 2 }};
        }

        Self {
            image_size,
            center_offset: Offset {
                x: compute_offset!(tile_region.x_tiles(), image_size.width),
                y: compute_offset!(tile_region.y_tiles(), image_size.height),
            },
            tile_span: Size {
                width: tile_region.x_tiles() * crate::TILE_SIZE,
                height: tile_region.y_tiles() * crate::TILE_SIZE,
            },
        }
    }
}

/// Latitude/Longitude based geometry of the map to be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapExtent {
    /// The center of the map, as a lat/lon point.
    pub center: Point,
    /// The full region of the map to be rendered, scaled by the image size
    /// to maintain the proper aspect ratio. (TODO)
    pub region: Region,
}

impl MapExtent {
    #[inline]
    pub fn new(region: Region) -> Self {
        Self {
            center: region.center(),
            region,
        }
    }
}

/// Tile geometry of the map to be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileExtent {
    pub zoom: Zoom,
    pub region: TileRegion,
}

impl TileExtent {
    pub fn new(region: &Region, image_size: Size<u32>) -> Self {
        let zoom = Zoom::from_region_and_size(&region, image_size);

        Self {
            zoom,
            region: TileRegion::from_region(&region, zoom),
        }
    }
}

impl MapGeometry {
    pub fn from_region(image_size: Size<u32>, region: Region) -> Self {
        let map = MapExtent::new(region);
        let tile = TileExtent::new(&map.region, image_size);
        let pixel = PixelExtent::new(image_size, &tile.region);

        Self { map, tile, pixel }
    }

    pub fn translate_point_pixel(&self, pt: geo::Point) -> Option<Offset> {
        let pt = self.translate_point(pt)?;

        let y = (pt.y - self.tile.region.y_min as f64) / self.tile.region.y_tiles() as f64;
        let x = (pt.x - self.tile.region.x_min as f64) / self.tile.region.x_tiles() as f64;

        Some(Offset {
            x: (x * self.pixel.image_size.width as f64).round() as i32,
            y: (y * self.pixel.image_size.height as f64).round() as i32,
        })
    }

    pub fn translate_point(&self, pt: geo::Point) -> Option<MapPoint> {
        if !self.map.region.contains(pt) {
            return None;
        }

        Some(MapPoint::new(pt, self.tile.zoom))
    }
}
