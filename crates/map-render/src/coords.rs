use std::f64::consts;

use geo::region::Region;
use geo::{Latitude, Longitude, Point};
use tiny_skia::Pixmap;

use crate::Error;
use crate::tiles::Tile;

pub(super) const TILE_SIZE: u32 = 1024;
pub(super) const TILE_SIZE_F64: f64 = TILE_SIZE as f64;

/// Maximum viewable lat/lon due to the mercator projection.
///
/// Derived from: `arctan(sinh(pi))`
pub(super) const MAX_LAT: geo::Latitude = geo::Latitude::new(85.0511);
/// Minimum viewable lat/lon due to the mercator projection.
pub(super) const MIN_LAT: geo::Latitude = geo::Latitude::new(-85.0511);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl Offset {
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

impl std::ops::Add for Offset {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Offset {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// A rectangular span region of tiles. (0, 0) is the top left of the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileRegion {
    pub(super) x_min: u32,
    pub(super) x_max: u32,
    pub(super) y_min: u32,
    pub(super) y_max: u32,
}

impl TileRegion {
    pub fn from_region(region: &Region, zoom: Zoom) -> Self {
        let lat_range = region.lat_range();
        let lon_range = region.lon_range();

        let x1 = from_longitude(*lon_range.start(), zoom).floor() as u32;
        let x2 = from_longitude(*lon_range.end(), zoom).floor() as u32;

        let y1 = from_latitude(*lat_range.start(), zoom).floor() as u32;
        let y2 = from_latitude(*lat_range.end(), zoom).floor() as u32;

        let (x_min, x_max) = if x1 > x2 { (x2, x1) } else { (x1, x2) };

        let (y_min, y_max) = if y1 > y2 { (y2, y1) } else { (y1, y2) };

        Self {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }

    pub const fn num_tiles(&self) -> u32 {
        self.y_tiles() * self.x_tiles()
    }

    pub const fn y_tiles(&self) -> u32 {
        (self.y_max - self.y_min) + 1
    }

    pub const fn x_tiles(&self) -> u32 {
        (self.x_max - self.x_min) + 1
    }
}

impl IntoIterator for TileRegion {
    type Item = Tile;
    type IntoIter = TileIter;

    fn into_iter(self) -> Self::IntoIter {
        TileIter {
            y: self.y_min,
            current_x: self.x_min,
            region: self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileIter {
    region: TileRegion,
    y: u32,
    current_x: u32,
}

impl Iterator for TileIter {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // check if we're still within the column bounds
            if self.y > self.region.y_max {
                return None;
            }

            // if we're still within a row,
            if self.current_x <= self.region.x_max {
                let x = self.current_x;
                self.current_x += 1;
                return Some(Tile { x, y: self.y });
            }

            // if not, move to the next row and repeat
            self.y += 1;
            self.current_x = self.region.x_min;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let x_tiles = self.region.x_tiles() as usize;
        let y_tiles = self.region.y_tiles() as usize;

        let total_count = x_tiles.saturating_mul(y_tiles) as usize;

        let full_cols_yielded = (self.y - self.region.y_min) as usize * x_tiles;
        let yielded_in_curr_row = (self.current_x - self.region.x_min) as usize;

        let remaining = total_count
            .saturating_sub(full_cols_yielded)
            .saturating_sub(yielded_in_curr_row);

        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for TileIter {}

/// Zoom levels.
///
/// Since mapbox recommends using 512x512 pixel tiles (versus the standard that specifies
/// 256x256 pixel tiles), the actual enum/integer repr of this type will be off by 1.
///
/// Use [`Zoom::as_mapbox_zoom`] to get the correct zoom value.
///
/// This [`Zoom`] is based on this, which leads to the following:
///
///  - The [`Zoom`] variants themselves follow the standard, so a zoom of 0 will cover the entire
///    earth (minus ~5 degrees from the top and bottom, due to the mercator projection).
///  - The integer repr of the enum variants is offset by one. This accounts for zoom difference
///    between 256x256 pixel and 512x512 pixel tiles.
///  - A consequence of this offset means we don't need to explicitely support the smallest zoom
///    level (22). A zoom of 21 with the larger tiles is the same thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Zoom {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Eleven = 11,
    Twelve = 12,
    Thirteen = 13,
    Fourteen = 14,
    Fifteen = 15,
    Sixteen = 16,
    Seventeen = 17,
    Eighteen = 18,
    Nineteen = 19,
    Twenty = 20,
    TwentyOne = 21,
}

impl Zoom {
    /// All zoom levels, ordered from a smaller zoom to a larger zoom
    /// (aka, larger area -> smaller area)
    ///
    /// ```
    /// #![feature(is_sorted)]
    /// # use map_render::Zoom;
    /// // Index matches the zoom level.
    /// assert_eq!(Zoom::ALL[0], Zoom::Zero);
    /// assert_eq!(Zoom::ALL[21], Zoom::TwentyOne);
    ///
    /// // And to ensure it's ordered:
    /// assert!(Zoom::ALL.is_sorted());
    /// ```
    pub const ALL: [Self; 22] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Eleven,
        Self::Twelve,
        Self::Thirteen,
        Self::Fourteen,
        Self::Fifteen,
        Self::Sixteen,
        Self::Seventeen,
        Self::Eighteen,
        Self::Nineteen,
        Self::Twenty,
        Self::TwentyOne,
    ];

    pub const fn as_mapbox_zoom(self) -> u32 {
        self as u32 + 1
    }

    pub const fn tiles_across(self) -> u32 {
        2_u32.pow(self.as_mapbox_zoom())
    }

    /// Get the width of a tile in longitudinal degrees for a given zoom.
    pub fn tile_width(self) -> f64 {
        360.0 / self.tiles_across() as f64
    }

    pub fn from_region_and_size(region: &Region, image_size: Size<u32>) -> Self {
        // we can't get partial tiles, so round up to the nearest full tile count
        let y_tiles = (image_size.height as f64 / TILE_SIZE_F64).ceil();
        let x_tiles = (image_size.width as f64 / TILE_SIZE_F64).ceil();

        let max_tiles = y_tiles.max(x_tiles);

        let y_extent = region.delta_lat().get();
        let x_extent = region.delta_lon().get();

        let max_extent = y_extent.min(x_extent);

        let delta_fn = |zoom: &Zoom| -> Option<i64> {
            let delta = max_extent - (zoom.tile_width() * max_tiles);
            println!("{zoom:?}: {delta}");

            // if under zero, we wont fit the entire region. if over zero, we'll have a buffer.
            // goal is to minimize
            if delta < 0.0 {
                None
            } else {
                // since the absolute value doesn't matter, we can scale up and turn
                // into an integer for [`Ord`]. We need to flip to negative so max_by_key
                // returns the closest to 0, but not under.
                let v = (delta * -1000.0) as i64;
                Some(v)
            }
        };

        Self::ALL.into_iter().max_by_key(delta_fn).unwrap() // Self::ALL is never empty, so the iterator __will__ return Some
    }

    pub fn to_region(&self, _center: geo::Point) -> Region {
        todo!()
    }

    /// Increments the zoom by an offset. If incrementing overflowed the valid zoom levels,
    /// returns [`false`].
    #[inline]
    pub fn increment(&mut self, by: i8) -> bool {
        if let Some(offset) = self.offset(by) {
            *self = offset;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn offset(self, by: i8) -> Option<Self> {
        Self::from_int((self as u8).saturating_add_signed(by))
    }

    #[inline]
    pub fn offset_saturating(self, by: i8) -> Self {
        Self::from_int((self as u8).saturating_add_signed(by).max(21))
            .expect("clamped between 0 and 21, which should always return Some")
    }

    #[inline]
    pub fn from_offset_int(int: u8) -> Option<Self> {
        // subtract by 1 to get the non-adjusted repr
        int.checked_sub(1).and_then(Self::from_int)
    }

    /// Gets the zoom from an integer. Returns the variant with the same name,
    /// AKA, this does __not__ account for the zoom level offset.
    /// [`Zoom::from_offset_int`] does this, if that behavior is desired.
    #[inline]
    pub fn from_int(int: u8) -> Option<Self> {
        match int {
            0 => Some(Self::Zero),
            1 => Some(Self::One),
            2 => Some(Self::Two),
            3 => Some(Self::Three),
            4 => Some(Self::Four),
            5 => Some(Self::Five),
            6 => Some(Self::Six),
            7 => Some(Self::Seven),
            8 => Some(Self::Eight),
            9 => Some(Self::Nine),
            10 => Some(Self::Ten),
            11 => Some(Self::Eleven),
            12 => Some(Self::Twelve),
            13 => Some(Self::Thirteen),
            14 => Some(Self::Fourteen),
            15 => Some(Self::Fifteen),
            16 => Some(Self::Sixteen),
            17 => Some(Self::Seventeen),
            18 => Some(Self::Eighteen),
            19 => Some(Self::Nineteen),
            20 => Some(Self::Twenty),
            21 => Some(Self::TwentyOne),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MapPoint<T = f64> {
    pub x: T,
    pub y: T,
    pub zoom: Zoom,
}

impl MapPoint {
    pub fn new(point: Point, zoom: Zoom) -> Self {
        Self {
            x: from_longitude(point.longitude(), zoom),
            y: from_latitude(point.latitude(), zoom),
            zoom,
        }
    }

    pub fn to_f32(self) -> MapPoint<f32> {
        MapPoint {
            x: self.x as f32,
            y: self.y as f32,
            zoom: self.zoom,
        }
    }

    pub fn to_tile(&self) -> Tile {
        Tile {
            x: self.x.floor() as u32,
            y: self.y.floor() as u32,
        }
    }
}

fn from_latitude(lat: Latitude, zoom: Zoom) -> f64 {
    let lat_rad = lat.get().to_radians();

    // reproject to spherical mercator
    let y = lat_rad.tan().asinh();

    // scale to 0-1 and shift origin
    let y = (1.0 - (y / consts::PI)) / 2.0;

    // scale by the number of tiles in 1 axis (via zoom) to get the tile location
    y * zoom.tiles_across() as f64
}

fn from_longitude(lon: Longitude, zoom: Zoom) -> f64 {
    let lon_rad = lon.get().to_radians();

    // projection matches, so skip to scaling
    // scale to 0-1 and shift origin
    let x = (1.0 + (lon_rad / consts::PI)) / 2.0;
    // scale by the number of tiles in 1 axis (via zoom) to get the tile location
    x * zoom.tiles_across() as f64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size<T> {
    pub height: T,
    pub width: T,
}

impl Size<u32> {
    pub const TILE: Self = Self {
        height: TILE_SIZE,
        width: TILE_SIZE,
    };

    pub fn ensure_tile_sized(self) -> Self {
        Self {
            height: self.height.max(TILE_SIZE),
            width: self.width.max(TILE_SIZE),
        }
    }

    pub fn build_empty_pixmap(self) -> Result<Pixmap, Error> {
        Pixmap::new(self.width, self.height).ok_or_else(|| Error::InvalidSize(self.into()))
    }
}

#[test]
fn test_pairs() {
    let test = TileRegion {
        x_min: 0,
        x_max: 2,
        y_min: 1,
        y_max: 3,
    };

    let pairs = test.into_iter().collect::<Vec<_>>();

    assert_eq!(pairs.as_slice(), &[
        // row 1
        Tile { x: 0, y: 1 },
        Tile { x: 1, y: 1 },
        Tile { x: 2, y: 1 },
        // row 2
        Tile { x: 0, y: 2 },
        Tile { x: 1, y: 2 },
        Tile { x: 2, y: 2 },
        // row 3
        Tile { x: 0, y: 3 },
        Tile { x: 1, y: 3 },
        Tile { x: 2, y: 3 },
    ]);

    let iter_len = test.into_iter().len();
    assert_eq!(pairs.len(), iter_len);
}
