use super::{GeoHash, Scale};

const MASK_AS: u64 = 0xaaaaaaaaaaaaaaaa;
const MASK_5S: u64 = 0x5555555555555555;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[inline]
const fn scale_shift(scale: Scale) -> usize {
    2 * (Scale::MAX as usize - scale as usize)
}

#[inline]
const fn move_latitude(hash: &mut GeoHash, north: bool) {
    let x = hash.hash & MASK_AS;
    let mut y = hash.hash & MASK_5S;

    let shift = scale_shift(hash.scale);
    let offset = MASK_AS >> shift;

    if north {
        y += offset + 1;
    } else {
        y |= offset;
        y -= offset - 1;
    }

    y &= MASK_5S >> shift;

    hash.hash = x | y;
}

#[inline]
const fn move_longitude(hash: &mut GeoHash, east: bool) {
    let mut x = hash.hash & MASK_AS;
    let y = hash.hash & MASK_5S;

    let shift = scale_shift(hash.scale);
    let offset = MASK_5S >> shift;

    if east {
        x = x + (offset + 1);
    } else {
        x = x | offset;
        x = x - (offset - 1);
    }

    x &= MASK_AS >> shift;

    hash.hash = x | y;
}

impl Direction {
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
    ];

    pub(super) const fn move_geohash(self, hash: &mut GeoHash) {
        match self {
            Self::North => move_latitude(hash, true),
            Self::NorthEast => {
                move_latitude(hash, true);
                move_longitude(hash, true);
            }
            Self::East => move_longitude(hash, true),
            Self::SouthEast => {
                move_latitude(hash, false);
                move_longitude(hash, true);
            }
            Self::South => move_latitude(hash, false),
            Self::SouthWest => {
                move_latitude(hash, false);
                move_longitude(hash, false);
            }
            Self::West => move_longitude(hash, false),
            Self::NorthWest => {
                move_latitude(hash, true);
                move_longitude(hash, false);
            }
        }
    }
}

pub struct NeighborIter {
    iter: std::array::IntoIter<Direction, 8>,
    center: GeoHash,
}
impl NeighborIter {
    #[inline]
    pub fn new(center: GeoHash) -> Self {
        Self {
            center,
            iter: Direction::ALL.into_iter(),
        }
    }
}

impl Iterator for NeighborIter {
    type Item = (Direction, GeoHash);

    fn next(&mut self) -> Option<Self::Item> {
        let dir = self.iter.next()?;
        let mut hash = self.center;

        dir.move_geohash(&mut hash);
        Some((dir, hash))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for NeighborIter {}

impl DoubleEndedIterator for NeighborIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let dir = self.iter.next_back()?;
        let mut hash = self.center;

        dir.move_geohash(&mut hash);
        Some((dir, hash))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Neighbors {
    pub north: GeoHash,
    pub north_east: GeoHash,
    pub east: GeoHash,
    pub south_east: GeoHash,
    pub south: GeoHash,
    pub south_west: GeoHash,
    pub west: GeoHash,
    pub north_west: GeoHash,
}

impl Neighbors {
    pub fn entire_region(&self) -> super::Region {
        let mut region = self.north_east.decode();
        region.merge(self.south_east.decode());
        region.merge(self.south_west.decode());
        region.merge(self.north_west.decode());
        region
    }
}
