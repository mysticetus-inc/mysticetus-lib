#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum WkbEndianness {
    #[default]
    Big,
    Little,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum WkbGeometryType {
    Geometry = 0,
    Point = 1,
    LineString = 2,
    Polygon = 3,
    MultiPoint = 4,
    MultiLineString = 5,
    MultiPolygon = 6,
    GeometryCollection = 7,
    CircularString = 8,
    CompoundCurve = 9,
    CurvePolygon = 10,
    MultiCurve = 11,
    MultiSurface = 12,
    Curve = 13,
    Surface = 14,
    PolyhedralSurface = 15,
    Tin = 16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum WkbPointType {
    // standard 2d point
    TwoDim = 0,
    // standard 3d point (Z WKB variant)
    ThreeDim = 1000,
    // standard 2d point, with an M value (M WKB variant)
    TwoDimM = 2000,
    // standard 3d point, with an M value (ZM WKB variant)
    ThreeDimM = 3000,
}
