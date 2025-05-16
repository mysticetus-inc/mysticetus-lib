use std::fmt;

use timestamp::Timestamp;

use crate::states::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressReport<'a> {
    pub(crate) state: State,
    pub(crate) progress: Progress,

    // the relevant user for this request
    pub(crate) uid: &'a str,

    pub(crate) started: Timestamp,
    pub(crate) last_updated: Timestamp,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub(crate) message: Option<&'a str>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub(crate) details: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result_file: Option<GcsLocation<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct GcsLocation<'a> {
    pub(crate) bucket: &'a str,
    pub(crate) path: &'a str,
}

impl<'a> GcsLocation<'a> {
    pub const fn new(bucket: &'a str, path: &'a str) -> Self {
        Self { bucket, path }
    }
}

fn is_none_or_empty(opt: &Option<&str>) -> bool {
    match opt {
        Some(s) => s.trim().is_empty(),
        None => true,
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Progress {
    current: usize,
    total: usize,
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = itoa::Buffer::new();

        f.write_str(buf.format(self.current))?;
        f.write_str("/")?;
        f.write_str(buf.format(self.total))
    }
}

#[inline]
const fn const_min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}
#[inline]
const fn const_max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

impl Progress {
    #[inline]
    pub const fn current(&self) -> usize {
        self.current
    }

    pub const fn increment(self, by: usize) -> Self {
        Self::new(self.current.saturating_add(by), self.total)
    }

    pub const fn update(self, new: usize) -> Self {
        Self::new(const_max(self.current, new), self.total)
    }

    #[inline]
    pub const fn total(&self) -> usize {
        self.total
    }

    #[inline]
    pub const fn new(current: usize, total: usize) -> Self {
        Self {
            current: const_min(current, total),
            total,
        }
    }

    pub fn as_percentage(&self) -> f64 {
        self.current as f64 / self.total as f64
    }

    #[inline]
    pub fn from_percentage(percen: f64) -> Self {
        Self {
            current: (percen.clamp(0.0, 1.0) * 1000.0) as usize,
            total: 1000,
        }
    }
}
