//! Internal utility functions/types.

use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::Table;

#[inline]
pub(crate) fn table_col_names<T: Table>() -> Vec<String> {
    let mut buf = Box::new_uninit_slice(T::COLUMNS.len());

    for i in 0..T::COLUMNS.len() {
        buf[i].write(T::COLUMNS[i].name.to_owned());
    }

    // SAFETY: 'buf' was initialized from the length of 'slice', then
    // each element was initialized with an owned version of each element in 'slice'.
    let table_names = unsafe { buf.assume_init().into_vec() };

    if T::NAME.contains("MasterFile") || T::NAME.contains("Station") {
        tracing::info!(message = "debug column order", table = T::NAME, table_names = ?table_names);
    }

    table_names
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

pub enum MaybeOwnedMut<'a, T> {
    Owned(T),
    MutRef(&'a mut T),
}

impl<'a, T> MaybeOwnedMut<'a, T> {
    #[inline]
    pub(crate) fn reborrow(&mut self) -> MaybeOwnedMut<'_, T> {
        MaybeOwnedMut::MutRef(&mut *self)
    }
}

impl<T: fmt::Debug> fmt::Debug for MaybeOwnedMut<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self, f)
    }
}

impl<T> From<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a mut T> for MaybeOwnedMut<'a, T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        Self::MutRef(value)
    }
}

impl<T> Deref for MaybeOwnedMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(owned) => owned,
            Self::MutRef(refer) => refer,
        }
    }
}

impl<T> DerefMut for MaybeOwnedMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(owned) => owned,
            Self::MutRef(refer) => refer,
        }
    }
}
