use crate::report::GcsLocation;

pub trait ReportState: private::Sealed + Sized {
    const DEFAULT: Self;

    type ResultFile<'a>: Copy + Into<Option<GcsLocation<'a>>>;

    fn to_report_state(&self) -> State;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum State {
    Running,
    Failed,
    Finished,
}

mod private {
    pub trait Sealed: Default + std::fmt::Debug {}
}

/// A regular update, which doesn't include reports about a finished operation or an error.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Update {
    _priv: (),
}

impl private::Sealed for Update {}

impl ReportState for Update {
    const DEFAULT: Self = Self { _priv: () };

    type ResultFile<'a> = NoResult;

    #[inline]
    fn to_report_state(&self) -> State {
        State::Running
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoResult;

impl<'a> From<NoResult> for Option<GcsLocation<'a>> {
    fn from(_: NoResult) -> Self {
        None
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Failed {
    _priv: (),
}

impl private::Sealed for Failed {}

impl ReportState for Failed {
    const DEFAULT: Self = Self { _priv: () };

    /// Optional file, since there may be something to point at
    type ResultFile<'a> = Option<GcsLocation<'a>>;

    fn to_report_state(&self) -> State {
        State::Failed
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Finished {
    _priv: (),
}

impl private::Sealed for Finished {}

impl ReportState for Finished {
    const DEFAULT: Self = Self { _priv: () };

    type ResultFile<'a> = GcsLocation<'a>;

    fn to_report_state(&self) -> State {
        State::Finished
    }
}
