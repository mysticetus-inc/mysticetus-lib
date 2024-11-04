use crate::Point;
use crate::region::Region;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Extent {
    /// The 'extent' of a single point.
    Point(Point),
    /// A region.
    Region(Region),
}

impl Extent {
    pub fn overlaps_region(&self, region: &Region) -> bool {
        match self {
            Self::Point(pt) => region.contains(*pt),
            Self::Region(reg) => region.overlaps_with(reg),
        }
    }
}

impl From<Point> for Extent {
    #[inline]
    fn from(pt: Point) -> Self {
        Self::Point(pt)
    }
}

impl From<Region> for Extent {
    #[inline]
    fn from(rg: Region) -> Self {
        Self::Region(rg)
    }
}
