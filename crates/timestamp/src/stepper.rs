use std::cmp::Ordering;
use std::iter::Step;

use super::{Duration, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepFrom {
    pub(crate) current: Timestamp,
    pub(crate) delta: Duration,
}

impl StepFrom {
    pub const fn stop_at(self, end: Timestamp) -> StepTo {
        StepTo {
            current: self.current,
            delta: self.delta,
            end,
        }
    }
}

impl PartialOrd for StepFrom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StepFrom {
    fn cmp(&self, other: &Self) -> Ordering {
        self.current.cmp(&other.current)
    }
}

impl From<std::ops::RangeFrom<Timestamp>> for StepFrom {
    fn from(range: std::ops::RangeFrom<Timestamp>) -> Self {
        Self {
            current: range.start,
            delta: Duration::SECOND,
        }
    }
}

impl Step for StepFrom {
    fn steps_between(_start: &Self, _end: &Self) -> Option<usize> {
        None
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let new = start
            .delta
            .checked_mul(count as i32)
            .and_then(|dur| start.current.add_duration_checked(dur))?;

        Some(Self {
            current: new,
            delta: start.delta,
        })
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let new = start
            .delta
            .checked_mul(count as i32)
            .and_then(|dur| start.current.sub_duration_checked(dur))?;

        Some(Self {
            current: new,
            delta: start.delta,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepTo {
    pub(crate) current: Timestamp,
    pub(crate) end: Timestamp,
    pub(crate) delta: Duration,
}

impl PartialOrd for StepTo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StepTo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.current.cmp(&other.current)
    }
}

impl Step for StepTo {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        let duration = (end.current - start.current).abs();
        let steps = duration
            .whole_nanoseconds()
            .checked_div(start.delta.whole_nanoseconds())?;

        match steps.abs().try_into() {
            Ok(uint) => Some(uint),
            Err(_) => Some(usize::MAX),
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let new_start = start
            .delta
            .checked_mul(count as i32)
            .and_then(|dur| start.current.add_duration_checked(dur))?;

        if new_start >= start.end {
            return None;
        }

        Some(Self {
            current: new_start,
            end: start.end,
            delta: start.delta,
        })
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let new_start = start
            .delta
            .checked_mul(count as i32)
            .and_then(|dur| start.current.sub_duration_checked(dur))?;

        if new_start >= start.end {
            return None;
        }

        Some(Self {
            current: new_start,
            end: start.end,
            delta: start.delta,
        })
    }
}

impl From<std::ops::Range<Timestamp>> for StepTo {
    fn from(range: std::ops::Range<Timestamp>) -> Self {
        Self {
            current: range.start,
            end: range.end,
            delta: Duration::SECOND,
        }
    }
}

impl From<std::ops::RangeFull> for StepTo {
    fn from(_: std::ops::RangeFull) -> Self {
        Self {
            current: Timestamp::MIN,
            end: Timestamp::MAX,
            delta: Duration::SECOND,
        }
    }
}

impl From<std::ops::RangeInclusive<Timestamp>> for StepTo {
    fn from(range: std::ops::RangeInclusive<Timestamp>) -> Self {
        Self {
            current: *range.start(),
            end: (*range.end()).add_duration(Duration::SECOND),
            delta: Duration::SECOND,
        }
    }
}
