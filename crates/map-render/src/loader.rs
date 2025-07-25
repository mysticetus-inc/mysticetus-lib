use std::sync::Arc;

use bytes::Bytes;
use gcp_auth_provider::Auth;
use tiny_skia::Pixmap;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::cache::TileCache;
use crate::coords::Zoom;
use crate::tiles::Tile;
use crate::{Config, Error, Map, MapGeometry};

const DEFUALT_TILE_CONCURRENCY: usize = 16;

#[derive(Debug, Clone)]
pub struct TileLoader {
    geometry: MapGeometry,
    client: reqwest::Client,
    cache: TileCache,
    config: Config,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CacheStats {
    misses: usize,
    hits: usize,
}

pub enum Cache {
    Hit,
    Miss,
}

impl CacheStats {
    fn add(&mut self, res: Cache) {
        match res {
            Cache::Hit => self.hits += 1,
            Cache::Miss => self.misses += 1,
        }
    }
}

impl TileLoader {
    pub fn new(client: reqwest::Client, auth: Auth, config: Config, geometry: MapGeometry) -> Self {
        Self {
            config,
            geometry,
            cache: TileCache::from_parts(client.clone(), auth),
            client,
        }
    }

    pub async fn build_base_map(self) -> Result<Map, Error> {
        self.build_base_map_with_concurrency(DEFUALT_TILE_CONCURRENCY)
            .await
    }

    pub async fn build_base_map_with_concurrency(self, concurrency: usize) -> Result<Map, Error> {
        let semaphore = Arc::new(Semaphore::new(concurrency.max(1)));

        // use an inner function, that way we only need to close the semaphore in one place
        async fn inner(loader: TileLoader, semaphore: &Arc<Semaphore>) -> Result<Map, Error> {
            let (tx, mut rx) = mpsc::unbounded_channel();

            for tile in loader.geometry.tile.region {
                loader.spawn_load_tile(tile, Arc::clone(semaphore), tx.clone());
            }

            // Drop the initial unused sender so it doesn't block the reciever
            drop(tx);

            let mut cache_stats = CacheStats::default();

            let mut image = loader.geometry.pixel.image_size.build_empty_pixmap()?;

            let ident = tiny_skia::Transform::identity();
            let paint = tiny_skia::PixmapPaint::default();

            while let Some(result) = rx.recv().await {
                let (tile, tile_image, cache) = result?;

                cache_stats.add(cache);

                let pos = tile.pixel_coords(&loader.geometry);

                image.draw_pixmap(pos.x, pos.y, tile_image.as_ref(), &paint, ident, None);
            }

            Ok(Map::from_parts(
                cache_stats,
                loader.geometry,
                image,
                loader.config,
            ))
        }

        match inner(self, &semaphore).await {
            Ok(base) => Ok(base),
            Err(err) => {
                semaphore.close();
                Err(err)
            }
        }
    }

    fn spawn_load_tile(
        &self,
        tile: Tile,
        semaphore: Arc<Semaphore>,
        result_tx: UnboundedSender<Result<(Tile, Pixmap, Cache), Error>>,
    ) {
        let fut = load_tile(
            self.config.clone(),
            self.client.clone(),
            self.cache.clone(),
            self.geometry.tile.zoom,
            tile,
        );

        tokio::spawn(async move {
            let permit = match semaphore.acquire_owned().await {
                Ok(permit) => permit,
                // if an error occurs while loading another tile, we close the semaphore
                // to indicate we should stop trying to load any more, as a pseudo-cancellation
                // technique.
                Err(_) => return,
            };

            let res = fut.await;

            drop(permit);

            let _ = result_tx.send(res);
        });
    }
}

/// Loads a single tile, trying first from the GCS cache and finally falling back to
/// grabbing a new one from mapbox.
async fn load_tile(
    config: Config,
    client: reqwest::Client,
    mut cache: TileCache,
    zoom: Zoom,
    tile: Tile,
) -> Result<(Tile, Pixmap, Cache), Error> {
    // try getting from the GCS cache first, fallback on mapbox.
    match cache.try_load_tile(&config, zoom, tile).await {
        Ok(Some((tile, map))) => return Ok((tile, map, Cache::Hit)),
        Ok(None) => (),
        Err(error) => error!(message = "error getting tile from cache", ?error),
    }

    let url = crate::util::build_tile_url(&config, zoom, tile);

    let (tile_pixmap, png_bytes) = load_tile_from_mapbox(client, url).await?;

    // Only try and upload if we know that the bytes are a valid PNG.
    cache.upload_tile(&config, zoom, tile, png_bytes);

    Ok((tile, tile_pixmap, Cache::Miss)) as crate::Result<(Tile, Pixmap, Cache)>
}

/// Gets the tile from mapbox, checks that it's a PNG, and parses it into a pixmap. Returns
/// the PNG bytes along with the pixmap to upload to GCS.
async fn load_tile_from_mapbox(
    client: reqwest::Client,
    url: String,
) -> Result<(Pixmap, Bytes), Error> {
    let resp = client.get(url).send().await?.error_for_status()?;

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|val| val.to_str().ok())
        .ok_or(crate::Error::MissingContentType)?
        .parse::<mime_guess::mime::Mime>()?;

    if content_type != mime_guess::mime::IMAGE_PNG {
        return Err(crate::Error::UnsupportedContentType(content_type));
    }

    let png_bytes = resp.bytes().await?;
    let tile = Pixmap::decode_png(&png_bytes)?;
    Ok((tile, png_bytes))
}
#[tokio::test]
async fn test_basic_render() -> crate::Result<()> {
    use shared::Shared;

    use crate::feature::point::{Circle, DrawPoint};

    const CONFIG: Config = Config {
        username: Shared::Static(env!("MAPBOX_USERNAME")),
        access_token: Shared::Static(env!("MAPBOX_ACCESS_TOKEN")),
        style_id: Shared::Static(env!("MAPBOX_ENC_STYLE_ID")),
    };

    tracing_subscriber::fmt::init();

    const MIN_LON: geo::Longitude = geo::Longitude::new(-121.364953);
    const MIN_LAT: geo::Latitude = geo::Latitude::new(44.021825);
    const MAX_LON: geo::Longitude = geo::Longitude::new(-121.255152);
    const MAX_LAT: geo::Latitude = geo::Latitude::new(44.095072);

    const MIDDLE_LON: geo::Longitude = geo::Longitude::new((MAX_LON.get() + MIN_LON.get()) / 2.0);
    const MIDDLE_LAT: geo::Latitude = geo::Latitude::new((MAX_LAT.get() + MIN_LAT.get()) / 2.0);

    const SIZE: crate::coords::Size<u32> = crate::coords::Size {
        height: 4096 / 2,
        width: 4096,
    };

    let region = geo::region::Region::try_from_iter([
        geo::Point::new(MIN_LON, MIN_LAT),
        geo::Point::new(MAX_LON, MAX_LAT),
    ])
    .unwrap();

    // make sure we're a real region
    assert!(!region.is_zero_area());

    let geom = MapGeometry::from_region(SIZE, region);

    let client = reqwest::Client::new();
    let auth = Auth::new_detect()
        .with_scopes(gcp_auth_provider::Scope::GcsAdmin)
        .await
        .unwrap();

    assert_eq!(auth.project_id().as_str(), "mysticetus-oncloud");

    let mut base = TileLoader::new(client, auth, CONFIG, geom)
        .build_base_map()
        .await?;

    let center = geo::Point::new(MIDDLE_LON, MIDDLE_LAT);

    let icon = Circle::new(tiny_skia::Color::from_rgba8(0, 255, 0, 255), 128.0)
        .with_border(tiny_skia::Color::BLACK, 16.0);

    let drawable = DrawPoint::builder().point(center).icon(&icon).build();

    base.draw(drawable)?;

    base.image.save_png("test.png")?;

    println!("{base:#?}");

    Ok(())
}
