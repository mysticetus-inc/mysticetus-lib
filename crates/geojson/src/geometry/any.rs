//! [`AnyCoordinate`] definition + impls
use std::fmt;

use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

use super::{
    Coordinate, GeometryType, LineString, MapCoordinate, NestedMapCoordinate, Point, Polygon,
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct WrongGeometry {
    found: GeometryType,
    expected: GeometryType,
}

impl WrongGeometry {
    pub fn found(&self) -> GeometryType {
        self.found
    }

    pub fn expected(&self) -> GeometryType {
        self.expected
    }
}

impl fmt::Display for WrongGeometry {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "expected '{}' coordinates, but found '{}' coordinates instead",
            self.expected, self.found,
        )
    }
}

impl std::error::Error for WrongGeometry {}

/// A coordinate type that can be a [`Point`], [`LineString`] or [`Polygon`]. Useful for
/// mixing many geometry types in a single [`FeatureCollection`]
///
/// [`FeatureCollection`]: [`crate::FeatureCollection`]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AnyCoordinate {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
}

impl From<Point> for AnyCoordinate {
    fn from(point: Point) -> Self {
        Self::Point(point)
    }
}

impl From<LineString> for AnyCoordinate {
    fn from(ls: LineString) -> Self {
        Self::LineString(ls)
    }
}

impl From<Polygon> for AnyCoordinate {
    fn from(poly: Polygon) -> Self {
        Self::Polygon(poly)
    }
}

impl From<Vec<Point>> for AnyCoordinate {
    fn from(ls: Vec<Point>) -> Self {
        Self::LineString(ls.into())
    }
}

impl From<Vec<LineString>> for AnyCoordinate {
    fn from(poly: Vec<LineString>) -> Self {
        Self::Polygon(poly.into())
    }
}

impl AnyCoordinate {
    pub fn as_point(&self) -> Result<&Point, WrongGeometry> {
        match self {
            Self::Point(point) => Ok(point),
            Self::LineString(_) => Err(WrongGeometry {
                found: GeometryType::LineString,
                expected: GeometryType::Point,
            }),
            Self::Polygon(_) => Err(WrongGeometry {
                found: GeometryType::Polygon,
                expected: GeometryType::Point,
            }),
        }
    }

    pub fn as_point_mut(&mut self) -> Result<&mut Point, WrongGeometry> {
        match self {
            Self::Point(point) => Ok(point),
            Self::LineString(_) => Err(WrongGeometry {
                found: GeometryType::LineString,
                expected: GeometryType::Point,
            }),
            Self::Polygon(_) => Err(WrongGeometry {
                found: GeometryType::Polygon,
                expected: GeometryType::Point,
            }),
        }
    }

    pub fn into_point(self) -> Result<Point, WrongGeometry> {
        match self {
            Self::Point(point) => Ok(point),
            Self::LineString(_) => Err(WrongGeometry {
                found: GeometryType::LineString,
                expected: GeometryType::Point,
            }),
            Self::Polygon(_) => Err(WrongGeometry {
                found: GeometryType::Polygon,
                expected: GeometryType::Point,
            }),
        }
    }

    pub fn as_line_string(&self) -> Result<&LineString, WrongGeometry> {
        match self {
            Self::LineString(line_string) => Ok(line_string),
            Self::Point(_) => Err(WrongGeometry {
                found: GeometryType::Point,
                expected: GeometryType::LineString,
            }),
            Self::Polygon(_) => Err(WrongGeometry {
                found: GeometryType::Polygon,
                expected: GeometryType::LineString,
            }),
        }
    }

    pub fn as_line_string_mut(&mut self) -> Result<&mut LineString, WrongGeometry> {
        match self {
            Self::LineString(line_string) => Ok(line_string),
            Self::Point(_) => Err(WrongGeometry {
                found: GeometryType::Point,
                expected: GeometryType::LineString,
            }),
            Self::Polygon(_) => Err(WrongGeometry {
                found: GeometryType::Polygon,
                expected: GeometryType::LineString,
            }),
        }
    }

    pub fn into_line_string(self) -> Result<LineString, WrongGeometry> {
        match self {
            Self::LineString(line_string) => Ok(line_string),
            Self::Point(_) => Err(WrongGeometry {
                found: GeometryType::Point,
                expected: GeometryType::LineString,
            }),
            Self::Polygon(_) => Err(WrongGeometry {
                found: GeometryType::Polygon,
                expected: GeometryType::LineString,
            }),
        }
    }

    pub fn as_polygon(&self) -> Result<&Polygon, WrongGeometry> {
        match self {
            Self::Polygon(poly) => Ok(poly),
            Self::Point(_) => Err(WrongGeometry {
                found: GeometryType::Point,
                expected: GeometryType::Polygon,
            }),
            Self::LineString(_) => Err(WrongGeometry {
                found: GeometryType::LineString,
                expected: GeometryType::Polygon,
            }),
        }
    }

    pub fn as_polygon_mut(&mut self) -> Result<&mut Polygon, WrongGeometry> {
        match self {
            Self::Polygon(poly) => Ok(poly),
            Self::Point(_) => Err(WrongGeometry {
                found: GeometryType::Point,
                expected: GeometryType::Polygon,
            }),
            Self::LineString(_) => Err(WrongGeometry {
                found: GeometryType::LineString,
                expected: GeometryType::Polygon,
            }),
        }
    }

    pub fn into_polygon(self) -> Result<Polygon, WrongGeometry> {
        match self {
            Self::Polygon(poly) => Ok(poly),
            Self::Point(_) => Err(WrongGeometry {
                found: GeometryType::Point,
                expected: GeometryType::Polygon,
            }),
            Self::LineString(_) => Err(WrongGeometry {
                found: GeometryType::LineString,
                expected: GeometryType::Polygon,
            }),
        }
    }
}

impl TryFrom<AnyCoordinate> for Point {
    type Error = WrongGeometry;

    fn try_from(value: AnyCoordinate) -> Result<Self, Self::Error> {
        value.into_point()
    }
}

impl TryFrom<AnyCoordinate> for LineString {
    type Error = WrongGeometry;

    fn try_from(value: AnyCoordinate) -> Result<Self, Self::Error> {
        value.into_line_string()
    }
}

impl TryFrom<AnyCoordinate> for Polygon {
    type Error = WrongGeometry;

    fn try_from(value: AnyCoordinate) -> Result<Self, Self::Error> {
        value.into_polygon()
    }
}

/// Helper wrapper for serializing [`AnyCoordinate`] as a map.
#[derive(Debug, Clone, Serialize)]
pub enum AnyMapCoordinate<'a> {
    Point(&'a Point),
    LineString(NestedMapCoordinate<'a, Point>),
    Polygon(NestedMapCoordinate<'a, LineString>),
}

impl Coordinate for AnyCoordinate {
    fn geometry_type(&self) -> GeometryType {
        match self {
            Self::Point(_) => GeometryType::Point,
            Self::LineString(_) => GeometryType::LineString,
            Self::Polygon(_) => GeometryType::Polygon,
        }
    }
}

impl MapCoordinate for AnyCoordinate {
    type MapCoordinate<'a> = AnyMapCoordinate<'a>;

    fn as_map_coordinate(&self) -> Self::MapCoordinate<'_> {
        match self {
            Self::Point(point) => AnyMapCoordinate::Point(point),
            Self::LineString(ls) => AnyMapCoordinate::LineString(ls.as_map_coordinate()),
            Self::Polygon(poly) => AnyMapCoordinate::Polygon(poly.as_map_coordinate()),
        }
    }
}

/// [`AnyCoordinate`], but with generic Point/LineString/Polygon types.
///
/// Used to deserialize/serialize custom geometry types, and provide efficient filtering of
/// ignored geometry types at deserialization time.
///
/// For example, if we need to deserialize a feature collection with mixed geometry types, but
/// if we only want the point variants, we can use [`PointCoordinateFilter`] (which is a type
/// alias to [`GenericCoordinate<Point, IgnoredAny, IgnoredAny>`]). This lets us ignore and skip
/// deserialization of the `LineString`/`Polygon` variants without throwing errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GenericCoordinate<Pnt, Line, Poly> {
    Point(Pnt),
    LineString(Line),
    Polygon(Poly),
}

/// A deserialization helper for deserializing a feature collection, but only deserializing
/// the coordinates for Point features.
///
/// The Point type defaults to [`Point`]
pub type PointCoordinateFilter<P = Point> = GenericCoordinate<P, IgnoredAny, IgnoredAny>;

/// A deserialization helper for deserializing a feature collection, but only deserializing
/// the coordinates for LineString features.
///
/// The LineString type defaults to [`LineString`]
pub type LineStringCoordinateFilter<L = LineString> = GenericCoordinate<IgnoredAny, L, IgnoredAny>;

/// A deserialization helper for deserializing a feature collection, but only deserializing
/// the coordinates for Polygon features.
///
/// The Polygon type defaults to [`Polygon`]
pub type PolygonCoordinateFilter<P = Polygon> = GenericCoordinate<IgnoredAny, IgnoredAny, P>;

impl<Pnt, Line, Poly> GenericCoordinate<Pnt, Line, Poly> {
    /// If the inner type is a [`Point`], returns [`Some(Pnt)`]. Otherwise returns [`None`]
    ///
    /// [`Point`]: [`Self::Point`]
    pub fn into_point(self) -> Option<Pnt> {
        match self {
            Self::Point(pnt) => Some(pnt),
            _ => None,
        }
    }

    /// If the inner type is a [`LineString`], returns [`Some(Line)`]. Otherwise returns [`None`]
    ///
    /// [`LineString`]: [`Self::LineString`]
    pub fn into_line_string(self) -> Option<Line> {
        match self {
            Self::LineString(line) => Some(line),
            _ => None,
        }
    }

    /// If the inner type is a [`Polygon`], returns [`Some(Poly)`]. Otherwise returns [`None`]
    ///
    /// [`Polygon`]: [`Self::Polygon`]
    pub fn into_polygon(self) -> Option<Poly> {
        match self {
            Self::Polygon(poly) => Some(poly),
            _ => None,
        }
    }

    /// Returns true if the inner type is a [`Point`].
    ///
    /// [`Point`]: [`Self::Point`]
    pub fn is_point(&self) -> bool {
        matches!(self, Self::Point(_))
    }

    /// Returns true if the inner type is a [`LineString`].
    ///
    /// [`LineString`]: [`Self::LineString`]
    pub fn is_line_string(&self) -> bool {
        matches!(self, Self::LineString(_))
    }

    /// Returns true if the inner type is a [`Polygon`].
    ///
    /// [`Polygon`]: [`Self::Polygon`]
    pub fn is_polygon(&self) -> bool {
        matches!(self, Self::Polygon(_))
    }
}

impl<Pnt, Line, Poly> Coordinate for GenericCoordinate<Pnt, Line, Poly> {
    fn geometry_type(&self) -> GeometryType {
        match self {
            Self::Point(_) => GeometryType::Point,
            Self::LineString(_) => GeometryType::LineString,
            Self::Polygon(_) => GeometryType::Polygon,
        }
    }
}
