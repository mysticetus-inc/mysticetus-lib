//! Rust port of <https://github.com/yinqiwen/geohash-int>, with added on base16 string repr.

use std::fmt;
use std::str::FromStr;

use super::region::Region;
use super::Point;
use crate::ang::Degrees;
use crate::lng_lat::CoordinateAxis;
use crate::{Latitude, Longitude};

mod encode_decode;
mod neighbor;

pub use neighbor::{Direction, NeighborIter, Neighbors};

#[rustfmt::skip]
const BASE16_TABLE: [u8; 16] = [
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', 
    b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p',
];

/// const-able binary searched specifically for BASE32_TABLE, pulled directly from the
/// `[T]::binary_search_by` method, and modified to not call intrinsics + use the u8
/// cmp rather than a compare function.
const fn binary_search_base16(target: u8) -> Option<usize> {
    let mut size = BASE16_TABLE.len();
    let mut left = 0;
    let mut right = size;
    while left < right {
        let mid = left + size / 2;

        // The reason why we use if/else control flow rather than match
        // is because match reorders comparison operations, which is perf sensitive.
        // This is x86 asm for u8: https://rust.godbolt.org/z/8Y8Pra.
        let a = BASE16_TABLE[mid];
        if a < target {
            left = mid + 1;
        } else if a > target {
            right = mid;
        } else {
            return Some(mid);
        }

        size = right - left;
    }

    None
}

pub const MAX_STR_LEN: usize = Scale::MAX as usize / CHARS_PER_BYTE;

static_assertions::const_assert_eq!(MAX_STR_LEN, 16);

const BITS_PER_CHAR: usize = 4;

const BOTTOM_BITS_MASK: u64 = (1 << BITS_PER_CHAR as u64) - 1;

const CHARS_PER_BYTE: usize = 8 / BITS_PER_CHAR;
static_assertions::const_assert_eq!(CHARS_PER_BYTE, 2);

/// The allowed scales, with 32 being the highest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scale {
    Four = 4,
    Eight = 8,
    Twelve = 12,
    Sixteen = 16,
    Twenty = 20,
    TwentyFour = 24,
    TwentyEight = 28,
    ThirtyTwo = 32,
}

const fn scale_error<T>(scale: Scale) -> Degrees
where
    T: ~const CoordinateAxis + ~const crate::GeometryNewType<Inner = f64>,
{
    let max = T::MAX.get();

    let div = 2_u64.pow(scale as _);

    Degrees::new(max / div as f64)
}

impl Scale {
    pub const MAX: Self = Self::ThirtyTwo;
    pub const MIN: Self = Self::Four;

    pub const ALL: [Self; 8] = [
        Self::Four,
        Self::Eight,
        Self::Twelve,
        Self::Sixteen,
        Self::Twenty,
        Self::TwentyFour,
        Self::TwentyEight,
        Self::ThirtyTwo,
    ];

    pub fn determine_ideal_scale(radius: Degrees) -> Option<Self> {
        let mut best_fit: Option<(Degrees, Self)> = None;

        for scale in Self::ALL.into_iter() {
            // lat error will always be the min of both, since it's range of values is smaller.
            let lat_err = scale.latitude_error();
            // skip if the radius is bigger, no matter what
            if radius > lat_err {
                continue;
            } else if radius == lat_err {
                // if we happen to hit the exact value, just return early
                return Some(scale);
            }

            let delta = lat_err - radius;

            match best_fit {
                Some((last_delta, scale)) if last_delta > delta => best_fit = Some((delta, scale)),
                None => best_fit = Some((delta, scale)),
                _ => (),
            }
        }

        best_fit.map(|(_, scale)| scale)
    }

    #[inline]
    pub fn latitude_error(self) -> Degrees {
        scale_error::<Latitude>(self)
    }

    #[inline]
    pub fn longitude_error(self) -> Degrees {
        scale_error::<Longitude>(self)
    }

    #[inline]
    pub const fn from_int(n: usize) -> Option<Self> {
        match n {
            4 => Some(Self::Four),
            8 => Some(Self::Eight),
            12 => Some(Self::Twelve),
            16 => Some(Self::Sixteen),
            20 => Some(Self::Twenty),
            24 => Some(Self::TwentyFour),
            28 => Some(Self::TwentyEight),
            32 => Some(Self::ThirtyTwo),
            _ => None,
        }
    }

    #[inline]
    pub const fn pop(self) -> Option<Self> {
        match self {
            Self::Four => None,
            Self::Eight => Some(Self::Four),
            Self::Twelve => Some(Self::Eight),
            Self::Sixteen => Some(Self::Twelve),
            Self::Twenty => Some(Self::Sixteen),
            Self::TwentyFour => Some(Self::Twenty),
            Self::TwentyEight => Some(Self::TwentyFour),
            Self::ThirtyTwo => Some(Self::TwentyEight),
        }
    }
}

/// Geohash
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeoHash {
    /// The encoded, interleaved hash.
    hash: u64,
    /// The scale of z-curve this hash coordinate lies on.
    /// Effectively the number of bits used to encode each coordinate.
    scale: Scale,
}

impl fmt::Display for GeoHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; MAX_STR_LEN];
        self.write_into(&mut buf).fmt(formatter)
    }
}

impl serde::Serialize for GeoHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = [0; MAX_STR_LEN];
        serializer.serialize_str(self.write_into(&mut buf))
    }
}

impl<'de> serde::Deserialize<'de> for GeoHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;

        Self::from_encoded(s).map_err(serde::de::Error::custom)
    }
}

impl GeoHash {
    pub fn from_encoded(hash_str: &str) -> Result<Self, GeoHashParseError> {
        fn normalize_geohash_char(index: usize, mut ch: char) -> Result<u8, GeoHashParseError> {
            ch.make_ascii_lowercase();

            if matches!(ch, 'a'..='p') {
                Ok(ch as u8)
            } else {
                Err(GeoHashParseError::invalid_char(index, ch))
            }
        }

        let len = hash_str.len();

        let scale = match Scale::from_int(len * 2) {
            Some(scale) => scale,
            None if len == 0 => return Err(GeoHashParseError::empty_string()),
            None => return Err(GeoHashParseError::exceeds_max_scale(len)),
        };

        let mut hash = 0;

        for (index, ch) in hash_str.char_indices() {
            let b = normalize_geohash_char(index, ch)?;
            let digit = binary_search_base16(b).unwrap();

            let bits = (digit as u64) & BOTTOM_BITS_MASK;

            // we need to offset the bits in reverse order, since the most significant
            // bits represents the first char, (aka lowest index).
            let offset = len - index - 1;
            hash |= bits << offset * BITS_PER_CHAR;
        }

        Ok(GeoHash { scale, hash })
    }

    #[inline]
    pub const fn encoded_len(&self) -> usize {
        self.scale as usize / CHARS_PER_BYTE
    }

    pub fn write_to_string(&self) -> String {
        let mut buf = [0; MAX_STR_LEN];
        self.write_into(&mut buf).to_owned()
    }

    /// Returns the hash, shifted so the leading bits are the most significant bits.
    #[inline]
    const fn get_shifted_hash(&self) -> u64 {
        let delta_scale = Scale::MAX as usize - self.scale as usize;
        self.hash << CHARS_PER_BYTE * delta_scale
    }

    #[inline]
    pub fn neighbor_iter(&self) -> NeighborIter {
        NeighborIter::new(*self)
    }

    #[inline]
    pub const fn neighbors(&self) -> Neighbors {
        Neighbors {
            north: self.neighbor(Direction::North),
            north_east: self.neighbor(Direction::NorthEast),
            east: self.neighbor(Direction::East),
            south_east: self.neighbor(Direction::SouthEast),
            south: self.neighbor(Direction::South),
            south_west: self.neighbor(Direction::SouthWest),
            west: self.neighbor(Direction::West),
            north_west: self.neighbor(Direction::NorthWest),
        }
    }

    #[inline]
    pub const fn neighbor(&self, direction: Direction) -> GeoHash {
        let mut copy = *self;
        direction.move_geohash(&mut copy);
        copy
    }

    pub fn write_into<'a>(&self, dst: &'a mut [u8]) -> &'a str {
        const SHIFT_TOP_TO_BOTTOM: u32 = u64::BITS - BITS_PER_CHAR as u32;

        let len = self.encoded_len();
        assert!(dst.len() >= len);

        let mut hash = self.get_shifted_hash();
        let mut i = 0;

        while i < len {
            let code = (hash >> SHIFT_TOP_TO_BOTTOM) & BOTTOM_BITS_MASK;
            dst[i] = BASE16_TABLE[code as usize];
            hash <<= BITS_PER_CHAR;
            i += 1;
        }

        // SAFETY: we wrote only ASCII bytes between 0..len, so this chunk is a valid UTF8 string.
        unsafe { std::str::from_utf8_unchecked(&dst[..len]) }
    }

    fn unscale(&mut self, target: Scale) {
        // we can only scale out, not in.
        let scale = self.scale as usize;
        let target_scale = target as usize;

        if scale <= target_scale {
            return;
        }

        let center = self.decode().center();
        *self = Self::new_scale(center, target);
    }

    pub fn pop(&mut self) {
        if let Some(popped) = self.scale.pop() {
            self.unscale(popped);
        }
    }

    pub fn new(pt: Point) -> Self {
        Self::new_scale(pt, Scale::MAX)
    }

    /// Encodes a point at the given scale. Scale __must__ be in the range '1..=32',
    /// or this will panic.
    pub fn new_scale(pt: Point, scale: Scale) -> Self {
        // normalizes a coordinate using its min and max to be in the range 0-1
        const fn normalize<T>(pt: T) -> f64
        where
            T: ~const CoordinateAxis + ~const crate::GeometryNewType<Inner = f64>,
        {
            let min: f64 = T::MIN.get();
            let max: f64 = T::MAX.get();

            (pt.get() - min) / (max - min)
        }

        let scale_to_u32: f64 = (1_u64 << scale as u32) as f64;

        // normalizes both coords to be between 0 and 1, then scales that to the
        // full range of u32's.
        let lat_scaled = normalize(pt.latitude()) * scale_to_u32;
        let lon_scaled = normalize(pt.longitude()) * scale_to_u32;

        let hash = interleave(lat_scaled as u32, lon_scaled as u32);

        Self { hash, scale }
    }

    pub fn decode(self) -> Region {
        const fn unscale<T>(int: u32, scale: Scale) -> (f64, f64)
        where
            T: ~const CoordinateAxis + ~const crate::GeometryNewType<Inner = f64>,
        {
            let min: f64 = T::MIN.get();
            let max: f64 = T::MAX.get();

            // We shift the input, so this hard-coded scale of 32 is valid for any scale here.
            let scale_by: f64 = (1_u64 << scale as u32) as f64;

            let dec_min = min + (int as f64 / scale_by) * (max - min);
            let dec_max = min + ((int as f64 + 1.0) / scale_by) * (max - min);

            (dec_min, dec_max)
        }

        let (lat, lon) = deinterleave(self.hash);

        let (lat_min, lat_max) = unscale::<Latitude>(lat, self.scale);
        let (lon_min, lon_max) = unscale::<Longitude>(lon, self.scale);

        let min_pt = Point::new(Longitude::new(lon_min), Latitude::new(lat_min));
        let max_pt = Point::new(Longitude::new(lon_max), Latitude::new(lat_max));

        let mut region = Region::from_point(min_pt);
        region.add_point(max_pt);
        region
    }

    pub fn decode_center(self) -> Point {
        self.decode().center()
    }
}

impl FromStr for GeoHash {
    type Err = GeoHashParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_encoded(s)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GeoHashParseError {
    inner: InnerParseError,
}

impl GeoHashParseError {
    #[inline]
    const fn invalid_char(index: usize, invalid_char: char) -> Self {
        Self {
            inner: InnerParseError::InvalidChar {
                invalid_char,
                index,
            },
        }
    }

    #[inline]
    const fn empty_string() -> Self {
        Self {
            inner: InnerParseError::EmptyString,
        }
    }

    #[inline]
    const fn exceeds_max_scale(len: usize) -> Self {
        Self {
            inner: InnerParseError::ExceedsMaxScale { len },
        }
    }
}

impl fmt::Debug for GeoHashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for GeoHashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl std::error::Error for GeoHashParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InnerParseError {
    InvalidChar { invalid_char: char, index: usize },
    EmptyString,
    ExceedsMaxScale { len: usize },
}

impl fmt::Display for InnerParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChar {
                invalid_char,
                index,
            } => {
                write!(
                    f,
                    "invalid geohash char '{invalid_char}' found at index {index}"
                )
            }
            Self::EmptyString => f.write_str("geohash cannot be parsed from an empty string"),
            Self::ExceedsMaxScale { len } => {
                write!(
                    f,
                    "geohash string cannot exceed {MAX_STR_LEN} bytes, found {len} bytes"
                )
            }
        }
    }
}

// bit shifting functions used in encoding and decoding

// spread takes a u32 and deposits its bits into the evenbit positions of a u64
#[inline]
const fn spread(x: u32) -> u64 {
    let mut new_x = x as u64;
    new_x = (new_x | (new_x << 16)) & 0x0000ffff0000ffff;
    new_x = (new_x | (new_x << 8)) & 0x00ff00ff00ff00ff;
    new_x = (new_x | (new_x << 4)) & 0x0f0f0f0f0f0f0f0f;
    new_x = (new_x | (new_x << 2)) & 0x3333333333333333;
    new_x = (new_x | (new_x << 1)) & 0x5555555555555555;

    new_x
}

// spreads the inputs, then shifts the y input and does a bitwise or to fill the remaining bits in x
#[inline]
const fn interleave(x: u32, y: u32) -> u64 {
    spread(x) | spread(y) << 1
}

// squashes the even bit positions of a u64 into a u32
#[inline]
const fn squash(x: u64) -> u32 {
    let mut new_x = x & 0x5555555555555555;
    new_x = (new_x | (new_x >> 1)) & 0x3333333333333333;
    new_x = (new_x | (new_x >> 2)) & 0x0f0f0f0f0f0f0f0f;
    new_x = (new_x | (new_x >> 4)) & 0x00ff00ff00ff00ff;
    new_x = (new_x | (new_x >> 8)) & 0x0000ffff0000ffff;
    new_x = (new_x | (new_x >> 16)) & 0x00000000ffffffff;
    new_x as u32
}

// uses the squash function to create a 32 from the even bits
// then shifts the input right and squashes to create a u32 from the odd bits
#[inline]
const fn deinterleave(x: u64) -> (u32, u32) {
    (squash(x), squash(x >> 1))
}

#[cfg(test)]
mod tests {
    use super::{Direction, GeoHash, GeoHashParseError, Latitude, Longitude, Point, Scale};
    use crate::ang::Degrees;

    const TEST_POINT: Point = Point::new(Longitude::new(120.54321), Latitude::new(-5.12345));
    const ENCODED: &str = "lhgcmpjeelmfpbed";

    #[test]
    fn test_basic_parse() -> Result<(), GeoHashParseError> {
        let enc = GeoHash::new(TEST_POINT);

        let enc_half = GeoHash::new_scale(TEST_POINT, Scale::Sixteen);
        let mut buf = [0; super::MAX_STR_LEN];

        let s = enc.write_into(&mut buf);
        assert_eq!(s, ENCODED);

        let s_short = enc_half.write_into(&mut buf);
        assert_eq!(s_short, &ENCODED[..s_short.len()]);

        let parsed = GeoHash::from_encoded(ENCODED)?;
        assert_eq!(parsed, enc);

        let uneven_parsed = GeoHash::from_encoded(&ENCODED[..ENCODED.len() - 4])?;

        let parsed_center = parsed.decode().center();
        let uneven_parsed_center = uneven_parsed.decode().center();

        let delta = parsed_center.angular_distance(&uneven_parsed_center);

        // missing the final 3 bytes should make the 2 points very close, but not exact.
        assert!(delta < Degrees::new(0.00001));
        assert_ne!(delta, Degrees::ZERO);
        Ok(())
    }

    #[test]
    fn test_basic_decode() -> Result<(), GeoHashParseError> {
        let hash = ENCODED.parse::<GeoHash>()?;

        let region = hash.decode();
        let parsed_center = region.center();

        let re_encoded_hash = GeoHash::new(parsed_center);
        let re_encoded_center = re_encoded_hash.decode().center();

        let angular_dist = parsed_center.angular_distance(&re_encoded_center);

        assert!(angular_dist < Degrees::new(f64::EPSILON));

        Ok(())
    }

    #[test]
    fn test_movement() {
        let mut hash = ENCODED.parse::<GeoHash>().unwrap();
        hash.pop();
        hash.pop();
        hash.pop();
        hash.pop();
        hash.pop();
        hash.pop();

        let region = hash.decode();

        let north = hash.neighbor(Direction::North);
        let original = north.neighbor(Direction::South);

        dbg!(region, north.decode(), original.decode());

        dbg!(hash, north, original);

        assert_eq!(hash, original);
    }

    #[test]
    fn test_error() {
        let lat_err = Scale::Four.latitude_error();
        let lon_err = Scale::Four.longitude_error();

        assert_eq!(lat_err, Degrees::new(5.625));
        assert_eq!(lon_err, Degrees::new(11.25));
    }
}
