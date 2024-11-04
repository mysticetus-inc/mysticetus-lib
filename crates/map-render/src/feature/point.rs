use tiny_skia::{
    Color, FillRule, Mask, Paint, PathBuilder, PixmapMut, PixmapPaint, PixmapRef, Transform,
};
use typed_builder::TypedBuilder;

use super::{DrawError, Drawable};
use crate::coords::Offset;

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct DrawPoint<'a, I: Icon + ?Sized> {
    point: geo::Point,
    #[builder(setter(into))]
    icon: &'a I,
}

/// Generic icon. Can be implemented by images (ex. [`PixmapRef`]), hardcoded shapes (ex.
/// [`Circle`]), etc.
pub trait Icon {
    /// The type of [`tiny_skia`] painter this icon needs to be drawn. For primitive shapes, this
    /// should probably be [`Paint`], and for icons this is likely going to be [`PixmapPaint`].
    type Paint<'a>: Default;

    fn draw_on(
        &self,
        opts: &mut IconOptions<'_, Self::Paint<'_>>,
        map: &mut PixmapMut<'_>,
    ) -> Result<(), DrawError>;
}

#[derive(Debug)]
pub struct IconOptions<'a, Paint: ?Sized> {
    pub transform: Transform,
    pub paint: &'a mut Paint,
    pub center: Offset,
    pub clip_mask: Option<&'a Mask>,
}

impl<I: Icon> Icon for &I {
    type Paint<'a> = I::Paint<'a>;

    fn draw_on(
        &self,
        opts: &mut IconOptions<'_, I::Paint<'_>>,
        map: &mut PixmapMut<'_>,
    ) -> Result<(), DrawError> {
        I::draw_on(self, opts, map)
    }
}

pub struct Rotate<I: Icon> {
    rotation: f32,
    icon: I,
}

impl<I: Icon> Icon for Rotate<I> {
    type Paint<'a> = I::Paint<'a>;

    fn draw_on(
        &self,
        opts: &mut IconOptions<'_, I::Paint<'_>>,
        map: &mut PixmapMut<'_>,
    ) -> Result<(), DrawError> {
        debug_assert!(
            opts.transform.is_identity(),
            "rotation overwrites existing transformations"
        );
        opts.transform = Transform::from_rotate(self.rotation);

        self.icon.draw_on(opts, map)
    }
}

pub struct Circle {
    fill_color: Color,
    radius_px: f32,
    border: Option<Border>,
}

impl Circle {
    pub const fn new(fill_color: Color, radius_px: f32) -> Self {
        Self {
            fill_color,
            radius_px,
            border: None,
        }
    }

    pub const fn with_border(mut self, border_color: Color, stroke_px: f32) -> Self {
        self.border = Some(Border {
            stroke_px,
            color: border_color,
        });
        self
    }
}

impl Icon for Circle {
    type Paint<'a> = Paint<'a>;

    fn draw_on(
        &self,
        opts: &mut IconOptions<'_, Paint<'_>>,
        map: &mut PixmapMut<'_>,
    ) -> Result<(), DrawError> {
        let center_x = opts.center.x as f32;
        let center_y = opts.center.y as f32;

        let outer_path = PathBuilder::from_circle(center_x, center_y, self.radius_px)
            .ok_or(DrawError::TinySkiaError)?;

        match self.border {
            Some(border) => {
                opts.paint.set_color(border.color);

                map.fill_path(
                    &outer_path,
                    &opts.paint,
                    FillRule::default(),
                    opts.transform,
                    opts.clip_mask,
                );

                let inner_path = tiny_skia::PathBuilder::from_circle(
                    center_x,
                    center_y,
                    self.radius_px - border.stroke_px,
                )
                .ok_or(DrawError::TinySkiaError)?;

                opts.paint.set_color(self.fill_color);

                map.fill_path(
                    &inner_path,
                    &opts.paint,
                    FillRule::default(),
                    opts.transform,
                    opts.clip_mask,
                );
            }
            None => {
                opts.paint.set_color(self.fill_color);

                map.fill_path(
                    &outer_path,
                    &opts.paint,
                    FillRule::default(),
                    opts.transform,
                    opts.clip_mask,
                );
            }
        }

        Ok(())
    }
}

impl Icon for PixmapRef<'_> {
    type Paint<'a> = PixmapPaint;

    fn draw_on(
        &self,
        opts: &mut IconOptions<'_, PixmapPaint>,
        map: &mut PixmapMut<'_>,
    ) -> Result<(), DrawError> {
        opts.paint.blend_mode = tiny_skia::BlendMode::SourceAtop;

        let center_x = opts.center.x - (self.width() / 2) as i32;
        let center_y = opts.center.y - (self.height() / 2) as i32;

        map.draw_pixmap(
            center_x,
            center_y,
            *self,
            opts.paint,
            opts.transform,
            opts.clip_mask,
        );

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    stroke_px: f32,
    color: Color,
}

impl Border {
    pub const fn new(stroke_px: f32) -> Self {
        Self {
            stroke_px,
            color: Color::BLACK,
        }
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl<I: ?Sized + Icon> Drawable for DrawPoint<'_, I> {
    fn draw(
        &mut self,
        geometry: &crate::MapGeometry,
        mut map: tiny_skia::PixmapMut<'_>,
    ) -> Result<(), DrawError> {
        let Some(center) = geometry.translate_point_pixel(self.point) else {
            return Ok(());
        };

        let mut clip_mask = tiny_skia::Mask::new(
            geometry.pixel.image_size.width,
            geometry.pixel.image_size.height,
        )
        .ok_or(DrawError::TinySkiaError)?;

        let mut pb = tiny_skia::PathBuilder::new();

        let width = geometry.pixel.image_size.width as f32;
        let height = geometry.pixel.image_size.height as f32;
        let rect =
            tiny_skia::Rect::from_xywh(0.0, 0.0, width, height).ok_or(DrawError::TinySkiaError)?;

        pb.push_rect(rect);

        pb.close();
        let path = pb.finish().ok_or(DrawError::TinySkiaError)?;

        clip_mask.fill_path(&path, FillRule::default(), true, Transform::identity());

        let mut paint = <I::Paint<'_> as Default>::default();

        let mut opts = IconOptions {
            transform: Transform::identity(),
            paint: &mut paint,
            center,
            clip_mask: Some(&clip_mask),
        };

        self.icon.draw_on(&mut opts, &mut map)
    }
}
