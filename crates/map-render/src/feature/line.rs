#![allow(dead_code)]
// Slightly zoomed in to give a bit of extra precision
const RESOLUTION_SCALE: f32 = 1.2;

pub struct DrawLine<I> {
    points: I,
    stroke: tiny_skia::Stroke,
}

impl<I> super::Drawable for DrawLine<I>
where
    I: Iterator<Item = geo::Point>,
{
    fn draw(
        &mut self,
        _geometry: &crate::MapGeometry,
        _map: tiny_skia::PixmapMut<'_>,
    ) -> Result<(), super::DrawError> {
        Ok(())
    }
}
