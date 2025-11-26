#![feature(const_trait_impl, let_chains)]

#[macro_use]
extern crate tracing;

mod cache;
mod coords;
mod error;
pub mod feature;
pub mod map_geometry;
use std::path::Path;

mod config;
mod util;

pub use config::Config;
pub use error::Error;
use loader::CacheStats;
pub use map_geometry::MapGeometry;
// mod layer;
mod loader;
pub use coords::Zoom;
use feature::{DrawError, Drawable, Feature};
pub use loader::TileLoader;

const TILE_SIZE: u32 = 1024;

#[cfg(feature = "benchmarking")]
pub mod bench_support {
    pub use crate::tiles::Tile;
    pub use crate::util::PartialUrl;
}

mod tiles;

pub type Result<T> = core::result::Result<T, Error>;

// Re-export for doctests.
#[cfg(test)]
pub use geo;
use tiny_skia::Pixmap;

#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    cache_stats: CacheStats,
    geometry: MapGeometry,
    image: Pixmap,
    config: Config,
}

impl Map {
    fn from_parts(
        cache_stats: CacheStats,
        geometry: MapGeometry,
        image: Pixmap,
        config: Config,
    ) -> Self {
        Self {
            cache_stats,
            geometry,
            image,
            config,
        }
    }

    pub fn render_to_bytes(self) -> Result<Vec<u8>> {
        self.image.encode_png().map_err(Error::from)
    }

    pub fn render_to_file<P>(self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.image.save_png(path.as_ref())?;
        Ok(())
    }

    pub fn draw<D: Drawable>(
        &mut self,
        mut drawable: D,
    ) -> core::result::Result<(), feature::DrawError> {
        drawable.draw(&self.geometry, self.image.as_mut())
    }

    pub fn draw_feature<F: Feature>(
        &mut self,
        feature: &F,
    ) -> core::result::Result<(), feature::DrawError> {
        if feature.extent().overlaps_region(&self.geometry.map.region) {
            self.draw(feature.as_drawable(&self.geometry))?;
        }

        Ok(())
    }

    pub fn draw_features<I>(&mut self, features: I) -> core::result::Result<(), DrawError>
    where
        I: IntoIterator,
        I::Item: Feature,
    {
        for feat in features {
            self.draw_feature(&feat)?;
        }

        Ok(())
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Map {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::http::header::{CONTENT_TYPE, HeaderValue};

        const PNG_HEADER: HeaderValue = HeaderValue::from_static("image/png");

        let image_bytes = match self.render_to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => return err.into_response(),
        };

        let headers = [(CONTENT_TYPE, PNG_HEADER)];

        (StatusCode::OK, headers, image_bytes).into_response()
    }
}

#[inline]
const fn n_digits(n: u32) -> usize {
    match n.checked_ilog10() {
        Some(n) => n as usize + 1,
        None => 1,
    }
}

fn sum_n_digits(d: &[u32]) -> usize {
    d.iter().copied().map(n_digits).sum()
}
