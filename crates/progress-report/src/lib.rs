mod builder;
mod driver;
mod report;
mod report_builder;
mod reporter;
mod states;

pub use report::{GcsLocation, Progress};
pub use report_builder::ReportBuilder;
pub use reporter::ProgressReporter;
pub use states::{Failed, Finished, ReportState, Update};

enum MaybeOwnedMut<'a, T> {
    MutRef(&'a mut T),
    Owned(T),
}

impl<T> std::ops::Deref for MaybeOwnedMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::MutRef(refer) => refer,
            Self::Owned(owned) => owned,
        }
    }
}

impl<T> std::ops::DerefMut for MaybeOwnedMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::MutRef(refer) => refer,
            Self::Owned(owned) => owned,
        }
    }
}
