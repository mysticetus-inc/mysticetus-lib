#![feature(hash_raw_entry)]

mod error;
use std::borrow::Cow;

pub use error::Error;

pub mod cell;
mod col_fmt;
mod content;
mod excel_file;
pub mod sheets;

pub use cell::Cell;
pub use content::{CellContent, ColHeader, Debug, DebugDisplay, Display};
pub use excel_file::{ExcelFile, HandleTemp};

/// Alias to [`core::result::Result`], with the [`Err`] variant already set to [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

// TODO: get real default
pub const DEFAULT_FONT_SIZE: f64 = 4.9;

pub const MIME_TYPE: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// The maximum sheet name length, in characters.
const MAX_SHEET_NAME_LEN: usize = 31;

/// Illegal characters in sheet names.
const INVALID_SHEET_NAME_CHARS: &[char] = &['[', ']', ':', '*', '?', '/', '\\'];

/// Takes a name, and modifies it to ensure it's a valid excel sheet name.
pub(crate) fn clean_sheet_name(mut s: Cow<'_, str>) -> Cow<'_, str> {
    // not allowed to start or end with '
    if s.starts_with('\'') {
        s.to_mut().remove(0);
    }

    if s.ends_with('\'') {
        s.to_mut().pop();
    }

    // verify we're not over the max sheet name length.
    if s.chars().count() > MAX_SHEET_NAME_LEN {
        let mut_ref = s.to_mut();
        mut_ref.truncate(MAX_SHEET_NAME_LEN - 3);
        mut_ref.push_str("...");
    }

    // replace any illegal characters.
    let mut start_from = 0;
    while let Some(offset) = s[start_from..].find(INVALID_SHEET_NAME_CHARS) {
        // SAFETY: all chars in INVALID_SHEET_NAME_CHARS are ascii,
        // and we're replacing one of them with another ascii replacement byte.
        unsafe {
            s.to_mut().as_bytes_mut()[start_from + offset] = b'|';
        }

        start_from += offset;
    }

    s
}
