use crate::tiles::Tile;
use crate::{MapGeometry, MapStyle};

mod base_urls {
    use crate::{MapStyle, Zoom};

    // +1 accounts for the 'mapbox' zoom levels going to 22, whereas Zoom
    // only goes to 21 explicitely. see docs on Zoom for details.
    const ZOOM_LEVELS: usize = Zoom::ALL.len() + 1;

    macro_rules! base_url {
        ($style:expr, $zoom:literal) => {{
            concat!(
                "https://api.mapbox.com/styles/v1/",
                env!("MAPBOX_USERNAME"),
                "/",
                $style,
                "/tiles/512/",
                stringify!($zoom),
                "/",
            )
        }};
    }

    macro_rules! base_urls {
        ($style:expr) => {{
            [
                base_url!($style, 0),
                base_url!($style, 1),
                base_url!($style, 2),
                base_url!($style, 3),
                base_url!($style, 4),
                base_url!($style, 5),
                base_url!($style, 6),
                base_url!($style, 7),
                base_url!($style, 8),
                base_url!($style, 9),
                base_url!($style, 10),
                base_url!($style, 11),
                base_url!($style, 12),
                base_url!($style, 13),
                base_url!($style, 14),
                base_url!($style, 15),
                base_url!($style, 16),
                base_url!($style, 17),
                base_url!($style, 18),
                base_url!($style, 19),
                base_url!($style, 20),
                base_url!($style, 21),
                base_url!($style, 22),
            ]
        }};
    }

    const BASE_STYLE_PARTIAL_URLS: [&'static str; ZOOM_LEVELS] =
        base_urls!(env!("MAPBOX_BASE_STYLE_ID"));

    const ENC_STYLE_PARTIAL_URLS: [&'static str; ZOOM_LEVELS] =
        base_urls!(env!("MAPBOX_ENC_STYLE_ID"));

    pub const fn get_partial_url(zoom: Zoom, style: MapStyle) -> &'static str {
        let index = zoom.as_mapbox_zoom() as usize;

        match style {
            MapStyle::Base => BASE_STYLE_PARTIAL_URLS[index],
            MapStyle::WithEnc => ENC_STYLE_PARTIAL_URLS[index],
        }
    }
}

/// Partial mapbox tile URL, containing everything up to the zoom level, which comes right before
/// the x/y tile indices. This also includes the style + mapbox username, etc. __Does__ include a
/// trailing '/'
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartialUrl(&'static str);

impl PartialUrl {
    pub(crate) fn new(geometry: &MapGeometry, style: MapStyle) -> Self {
        Self(base_urls::get_partial_url(geometry.tile.zoom, style))
    }

    #[cfg(feature = "benchmarking")]
    pub fn bench_new(zoom: crate::Zoom, style: MapStyle) -> Self {
        Self(base_urls::get_partial_url(zoom, style))
    }

    pub fn complete_url(&self, tile: Tile) -> String {
        const MULT_PARAM: &str = "@2x";
        const ACCESS_TOKEN_QP: &str = "?access_token=";

        let mut buf = itoa::Buffer::new();

        let capacity = self.0.len()
            + crate::n_digits(tile.x) + 1 // x + trailing slash
            + crate::n_digits(tile.y) // y
            + MULT_PARAM.len()
            + ACCESS_TOKEN_QP.len()
            + crate::MAPBOX_ACCESS_TOKEN.len(); // token query param + token itself

        let mut dst = String::with_capacity(capacity);

        dst.push_str(self.0);
        dst.push_str(buf.format(tile.x));
        dst.push_str("/");
        dst.push_str(buf.format(tile.y));
        dst.push_str(MULT_PARAM);
        dst.push_str(ACCESS_TOKEN_QP);
        dst.push_str(crate::MAPBOX_ACCESS_TOKEN);

        debug_assert_eq!(dst.capacity(), capacity, "capacity math is off");

        dst
    }
}
