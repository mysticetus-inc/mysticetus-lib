use std::cmp::Ordering;

use tiny_skia::PixmapMut;

use crate::MapGeometry;

pub mod line;
pub mod point;
pub mod polygon;

#[derive(Debug, Clone, thiserror::Error)]
pub enum DrawError {
    #[error("tiny_skia error")]
    TinySkiaError,
}

pub trait Drawable {
    fn draw(&mut self, geometry: &MapGeometry, map: PixmapMut<'_>) -> Result<(), DrawError>;
}

impl<T: Drawable> Drawable for Option<T> {
    fn draw(&mut self, geometry: &MapGeometry, map: PixmapMut<'_>) -> Result<(), DrawError> {
        if let Some(drawable) = self {
            drawable.draw(geometry, map)
        } else {
            Ok(())
        }
    }
}

impl<F: Feature> Drawable for F {
    fn draw(&mut self, geometry: &MapGeometry, map: PixmapMut<'_>) -> Result<(), DrawError> {
        self.as_drawable(geometry).draw(geometry, map)
    }
}

pub trait Feature {
    type Drawable<'a>: Drawable
    where
        Self: 'a;

    fn as_drawable(&self, map_geometry: &MapGeometry) -> Self::Drawable<'_>;

    fn extent(&self) -> geo::extent::Extent;
}

impl<T: Feature> Feature for &T {
    type Drawable<'a>
        = T::Drawable<'a>
    where
        Self: 'a;

    fn extent(&self) -> geo::extent::Extent {
        T::extent(self)
    }

    fn as_drawable(&self, map_geometry: &MapGeometry) -> Self::Drawable<'_> {
        T::as_drawable(self, map_geometry)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ZLevel {
    #[default]
    InOrder,
    Front(usize),
    Back(usize),
}

impl PartialOrd for ZLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ZLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        match (*self, *other) {
            (Self::InOrder, Self::InOrder) => Ordering::Equal,
            (Self::InOrder, Self::Front(_)) => Ordering::Greater,
            (Self::InOrder, Self::Back(_)) => Ordering::Less,
            (Self::Front(a), Self::Front(b)) => a.cmp(&b),
            (Self::Front(_), _) => Ordering::Less,
            (Self::Back(_), Self::InOrder | Self::Front(_)) => Ordering::Greater,
            (Self::Back(a), Self::Back(b)) => b.cmp(&a),
        }
    }
}
