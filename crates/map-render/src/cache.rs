use bytes::Bytes;
use gcp_auth_provider::Auth;
use tiny_skia::Pixmap;

use crate::coords::Zoom;
use crate::tiles::Tile;
use crate::{Config, Error};

const TILE_CACHE_BUCKET: &str = "mysticetus-mapbox-tile-cache";

const TILE_EXT: &str = ".png";

#[derive(Debug, Clone)]
pub struct TileCache {
    gcs: small_gcs::BucketClient,
}

impl TileCache {
    pub fn from_client(gcs: small_gcs::BucketClient) -> Self {
        Self { gcs }
    }

    pub fn from_parts(client: reqwest::Client, auth: Auth) -> Self {
        Self::from_client(small_gcs::BucketClient::from_parts(
            client,
            auth,
            TILE_CACHE_BUCKET.into(),
        ))
    }

    pub async fn try_load_tile(
        &mut self,
        config: &Config,
        zoom: Zoom,
        tile: Tile,
    ) -> Result<Option<(Tile, Pixmap)>, Error> {
        let path = build_tile_path(&config.style_id, zoom, tile);

        let bytes = match self.gcs.read(&path).content_to_bytes_opt(1234).await? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let map = Pixmap::decode_png(&bytes)?;

        Ok(Some((tile, map)))
    }

    pub fn upload_tile(&self, config: &Config, zoom: Zoom, tile: Tile, png_bytes: Bytes) {
        let mut gcs = self.gcs.clone();
        let path = build_tile_path(&config.style_id, zoom, tile);

        tokio::spawn(async move {
            if let Err(error) = gcs
                .write(&path)
                .content_len(png_bytes.len() as u64)
                .mime_type(mime_guess::mime::IMAGE_PNG)
                .upload(png_bytes)
                .await
            {
                error!(message = "whoops lmao", ?error);
            } else {
                info!(message = "added tile to the GCS cache", path);
            }
        });
    }
}

fn build_tile_path(style_id: &str, zoom: Zoom, tile: Tile) -> String {
    let mut buf = itoa::Buffer::new();

    const PATH_SEP_LEN: usize = 4;
    const UNDERSCORE_SEP_LEN: usize = 3;

    let component_len =
        crate::sum_n_digits(&[zoom.as_mapbox_zoom(), tile.x, tile.y]) + style_id.len();

    let cap = (2 * component_len) + PATH_SEP_LEN + UNDERSCORE_SEP_LEN + TILE_EXT.len();

    let mut dst = String::with_capacity(cap);

    macro_rules! push_parts {
        ($sep:literal => $trailing:expr) => {
            // leading path
            dst.push_str(style_id);
            dst.push_str($sep);
            dst.push_str(buf.format(zoom.as_mapbox_zoom()));
            dst.push_str($sep);
            dst.push_str(buf.format(tile.x));
            dst.push_str($sep);
            dst.push_str(buf.format(tile.y));
            dst.push_str($trailing);
        };
    }

    // path (+ a trailing /)
    push_parts!("/" => "/");

    // filename w/ extension
    push_parts!("_" => TILE_EXT);

    debug_assert_eq!(dst.len(), cap);
    dst
}
