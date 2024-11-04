/// Casts a slice to an array.
///
/// The length of the slice **MUST** be equal to N, or undefined behavior is more than likely.
///
/// Meant to be paired with the [`array_slice!`] macro, which creates the subslice + 'N' in a way
/// that satisfies this invariant.
#[inline]
pub(crate) const unsafe fn to_array<const N: usize, T>(slice: &[T]) -> &[T; N] {
    unsafe { &*slice.as_ptr().cast::<[T; N]>() }
}

#[const_trait]
pub trait ArraySlice<T> {
    fn leading<const N: usize>(&self) -> &[T; N];
    fn leading_mut<const N: usize>(&mut self) -> &mut [T; N];

    fn trailing<const N: usize>(&self) -> &[T; N];
    fn trailing_mut<const N: usize>(&mut self) -> &mut [T; N];

    fn slice<const OFFSET: usize, const N: usize>(&self) -> &[T; N];
    fn slice_mut<const OFFSET: usize, const N: usize>(&mut self) -> &mut [T; N];
}

impl<T> const ArraySlice<T> for [T] {
    #[inline]
    fn slice<const OFFSET: usize, const N: usize>(&self) -> &[T; N] {
        assert!(self.len() >= N + OFFSET);

        // SAFETY: we assert that N + OFFSET is equal or less than LEN.
        unsafe { &*(self.as_ptr().offset(OFFSET as isize) as *const [T; N]) }
    }

    #[inline]
    fn slice_mut<const OFFSET: usize, const N: usize>(&mut self) -> &mut [T; N] {
        assert!(self.len() >= N + OFFSET);

        // SAFETY: we assert that N is equal or less than LEN.
        unsafe { &mut *(self.as_mut_ptr().offset(OFFSET as isize) as *mut [T; N]) }
    }

    #[inline]
    fn leading<const N: usize>(&self) -> &[T; N] {
        self.slice::<0, N>()
    }

    #[inline]
    fn leading_mut<const N: usize>(&mut self) -> &mut [T; N] {
        self.slice_mut::<0, N>()
    }

    #[inline]
    fn trailing<const N: usize>(&self) -> &[T; N] {
        assert!(self.len() >= N);

        // SAFETY: we assert that len >= N.
        unsafe { &*(self.as_ptr().offset((self.len() - N) as isize) as *const [T; N]) }
    }

    #[inline]
    fn trailing_mut<const N: usize>(&mut self) -> &mut [T; N] {
        assert!(self.len() >= N);

        // SAFETY: we assert that len >= N.
        unsafe { &mut *(self.as_mut_ptr().offset((self.len() - N) as isize) as *mut [T; N]) }
    }
}
