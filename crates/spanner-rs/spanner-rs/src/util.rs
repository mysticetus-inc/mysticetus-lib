//! Internal utility functions/types.

use std::fmt;
use std::ops::Deref;

use crate::Table;

#[inline]
pub(crate) fn table_col_names<T: Table>() -> Vec<String> {
    let mut buf = Box::new_uninit_slice(T::COLUMNS.len());

    for i in 0..T::COLUMNS.len() {
        buf[i].write(T::COLUMNS[i].name.to_owned());
    }

    // SAFETY: 'buf' was initialized from the length of 'slice', then
    // each element was initialized with an owned version of each element in 'slice'.
    unsafe { buf.assume_init().into_vec() }
}

/*
#[inline]
pub(crate) fn slice_to_owned(slice: &[&str]) -> Vec<String> {
    let mut buf = Box::new_uninit_slice(slice.len());

    for i in 0..slice.len() {
        buf[i].write(slice[i].to_owned());
    }

    // SAFETY: 'buf' was initialized from the length of 'slice', then
    // each element was initialized with an owned version of each element in 'slice'.
    unsafe { buf.assume_init().into_vec() }
}

#[inline]
pub(crate) fn slice_to_buf(bytes: &[u8]) -> Vec<u8> {
    let mut dst = Box::<[u8]>::new_uninit_slice(bytes.len());

    dst.write_copy_of_slice(bytes);
    // SAFETY: the above call to 'write_slice' filled 'dst' with 'bytes',
    // and since they share the same length 'dst' is fully initialized.
    unsafe { dst.assume_init().into_vec() }
}
*/

pub enum MaybeOwned<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<'a, T> MaybeOwned<'a, T> {
    #[inline]
    pub(crate) fn reborrow(&self) -> MaybeOwned<'_, T> {
        MaybeOwned::Borrowed(self)
    }
}

impl<T: fmt::Debug> fmt::Debug for MaybeOwned<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self, f)
    }
}

impl<T> From<T> for MaybeOwned<'_, T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    #[inline]
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

impl<T> Deref for MaybeOwned<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(owned) => owned,
            Self::Borrowed(refer) => refer,
        }
    }
}
