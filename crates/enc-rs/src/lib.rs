#![feature(
    const_for,
    is_sorted,
    seek_stream_len,
    maybe_uninit_uninit_array,
    const_maybe_uninit_uninit_array,
    maybe_uninit_slice,
    maybe_uninit_write_slice,
    const_trait_impl,
    slice_pattern,
    const_try,
    buf_read_has_data_left,
    const_mut_refs,
    hash_set_entry
)]

pub mod complex;
pub mod iso8211;
mod reader;
mod tag;
pub use tag::Tag;
mod s57;
mod slicing;

pub use reader::Iso8211Reader;

pub mod error;
pub mod utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[repr(u8)]
pub enum Endianness {
    #[serde(rename = "le")]
    Little = b'b',
    #[serde(rename = "be")]
    Big = b'B',
}

#[const_trait]
pub(crate) trait FromByteArray<const N: usize>: Sized {
    type Error;

    fn from_byte_array(array: &[u8; N]) -> Result<Self, Self::Error>;
}

#[const_trait]
pub(crate) trait FromBytes: Sized {
    type Error;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>;
}

pub(crate) trait FromByte: Sized {
    type Error;

    fn from_byte(byte: u8) -> Result<Self, Self::Error>;

    fn from_byte_optional(byte: u8, optional: &[u8]) -> Result<Option<Self>, Self::Error> {
        let mut opt_idx = 0;
        while opt_idx < optional.len() {
            if optional[opt_idx] == byte {
                return Ok(None);
            }
            opt_idx += 1;
        }

        match Self::from_byte(byte) {
            Ok(val) => Ok(Some(val)),
            Err(err) => Err(err),
        }
    }
}
