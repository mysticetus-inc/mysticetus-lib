use std::cmp::Ordering;

/// Wrapper around [`f64`] that uses [`f64::total_cmp`] in the [`PartialEq`], [`Eq`],
/// [`PartialOrd`] and [`Ord`] trait impls, overriding the default [`f64`] versions.
#[derive(Debug, Clone, Copy, Default)]
pub struct TotalCmp(pub f64);

impl From<TotalCmp> for f64 {
    #[inline]
    fn from(t: TotalCmp) -> Self {
        t.0
    }
}

impl From<f64> for TotalCmp {
    #[inline]
    fn from(f: f64) -> Self {
        TotalCmp(f)
    }
}

impl PartialEq for TotalCmp {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0).is_eq()
    }
}

impl PartialEq<f64> for TotalCmp {
    #[inline]
    fn eq(&self, other: &f64) -> bool {
        self.0.total_cmp(other).is_eq()
    }
}

impl PartialEq<TotalCmp> for f64 {
    #[inline]
    fn eq(&self, other: &TotalCmp) -> bool {
        self.total_cmp(&other.0).is_eq()
    }
}

impl PartialOrd for TotalCmp {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.total_cmp(&other.0))
    }
}

impl PartialOrd<f64> for TotalCmp {
    #[inline]
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        Some(self.0.total_cmp(other))
    }
}

impl PartialOrd<TotalCmp> for f64 {
    #[inline]
    fn partial_cmp(&self, other: &TotalCmp) -> Option<Ordering> {
        Some(self.total_cmp(&other.0))
    }
}

impl Eq for TotalCmp {}

impl Ord for TotalCmp {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}
