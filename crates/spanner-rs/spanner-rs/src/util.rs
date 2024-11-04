//! Internal utility functions.

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

    std::mem::MaybeUninit::copy_from_slice(&mut dst, bytes);

    // SAFETY: the above call to 'write_slice' filled 'dst' with 'bytes',
    // and since they share the same length 'dst' is fully initialized.
    unsafe { dst.assume_init().into_vec() }
}
