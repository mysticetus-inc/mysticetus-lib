use crate::tiles::Tile;
use crate::{Config, Zoom};

/*
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
*/

pub(crate) fn build_tile_url(config: &Config, zoom: Zoom, tile: Tile) -> String {
    const BASE_URL: &str = "https://api.mapbox.com/styles/v1";
    const PATH_PART: &str = "tiles/512";
    const MULT_PARAM: &str = "@2x";
    const ACCESS_TOKEN_QP: &str = "?access_token=";

    let mut buf = itoa::Buffer::new();

    let capacity = BASE_URL.len() + 1 // base + trailing slash
            + config.username.len() + 1 // username + slash
            + config.style_id.len() + 1 // style id + slash
            + PATH_PART.len() + 1 // const path param + trailing slash
            + crate::n_digits(zoom as u8 as u32) + 1 // zoom len + trailing slash
            + crate::n_digits(tile.x) + 1 // x + trailing slash
            + crate::n_digits(tile.y) // y
            + MULT_PARAM.len()
            + ACCESS_TOKEN_QP.len()
            + config.access_token.len(); // token query param + token itself

    let mut dst = String::with_capacity(capacity);

    macro_rules! push_w_slash {
        ($value:expr) => {
            dst.push_str($value);
            dst.push('/');
        };
    }

    push_w_slash!(BASE_URL);
    push_w_slash!(&config.username);
    push_w_slash!(&config.style_id);
    push_w_slash!(PATH_PART);
    push_w_slash!(buf.format(zoom as u8));
    push_w_slash!(buf.format(tile.x));

    dst.push_str(buf.format(tile.y));
    dst.push_str(MULT_PARAM);
    dst.push_str(ACCESS_TOKEN_QP);
    dst.push_str(&config.access_token);

    debug_assert_eq!(
        dst.capacity(),
        capacity,
        "map_render::util::build_tile_url capacity math is off"
    );

    dst
}
