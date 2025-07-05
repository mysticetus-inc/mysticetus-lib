use std::borrow::Cow;
use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer};

/// A path within a data structure.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Path {
    #[default]
    /// At the root of the data structure.
    Root,
    /// A path to a nested field.
    Nested {
        /// The [`Segment`]s making up the nested path.
        segments: Vec<Segment>,
    },
}

pub struct ErrorPath {
    inner: Cell<Option<Path>>,
    has_been_set: Cell<bool>,
}

impl Clone for ErrorPath {
    fn clone(&self) -> Self {
        let inner = self.inner.take();
        self.inner.set(inner.clone());

        Self {
            inner: Cell::new(inner),
            has_been_set: Cell::new(self.has_been_set.get()),
        }
    }
}

impl fmt::Debug for ErrorPath {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("ErrorPath").finish_non_exhaustive()
    }
}

impl ErrorPath {
    #[inline]
    pub(crate) fn new<E>(path: E) -> Self
    where
        E: Into<Option<Path>>,
    {
        Self {
            inner: Cell::new(path.into()),
            has_been_set: Cell::new(false),
        }
    }

    pub(crate) fn take(&self) -> Option<Path> {
        if self.has_been_set.get() {
            self.inner.take()
        } else {
            None
        }
    }

    pub(crate) fn set(&self, seg_track: &Track<'_>) {
        // set to true. if already true, bail so we don't overwrite with a less nested path.
        if self.has_been_set.replace(true) {
            return;
        }

        let path = if let Some(mut root) = self.inner.take() {
            root.extend_from_track(seg_track);
            root
        } else {
            Path::from(seg_track)
        };

        self.inner.set(Some(path));
    }
}

impl Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.is_empty() {
            serializer.serialize_str("/")
        } else {
            serializer.collect_str(self)
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let segments = match self {
            Self::Nested { segments } => segments,
            Self::Root => return write!(formatter, "."),
        };

        fn should_insert_period_separator(next: Option<&Segment>) -> bool {
            match next {
                Some(Segment::Unknown) => true,
                Some(Segment::Map(key)) => !key.contains(char::is_whitespace),
                _ => false,
            }
        }

        let mut iter = segments.iter().peekable();

        while let Some(seg) = iter.next() {
            match seg {
                Segment::Index(idx) => write!(formatter, "[{idx}]")?,
                Segment::Map(key) => {
                    if key.contains(char::is_whitespace) {
                        write!(formatter, "[\"{key}\"]")?;
                    } else {
                        formatter.write_str(key)?;
                    }
                }
                Segment::Unknown => formatter.write_str("?")?,
            }

            // add a '.' separator if we have a map key or unknown segment.
            // this skips if we're at the end, or are an index, which prevents the array brackets
            // from being prefixed with a '.'
            if should_insert_period_separator(iter.peek().copied()) {
                formatter.write_str(".")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ParsePathError {
    NoClosingBracket(usize),
    NoOpeningBracket(usize),
    InvalidIndex(std::num::ParseIntError),
}

impl fmt::Display for ParsePathError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoClosingBracket(index) => {
                write!(
                    formatter,
                    "opening bracket at index {index} has no closing bracket"
                )
            }
            Self::NoOpeningBracket(index) => {
                write!(
                    formatter,
                    "found an unmatched indexer closing bracket at index {index}"
                )
            }
            Self::InvalidIndex(err) => {
                write!(formatter, "found an index that isnt a valid uint: {err}")
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Separator {
    Dot = b'.',
    OpenBracket = b'[',
    CloseBracket = b']',
}

impl Separator {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'.' => Some(Separator::Dot),
            b'[' => Some(Separator::OpenBracket),
            b']' => Some(Separator::CloseBracket),
            _ => None,
        }
    }
}

impl FromStr for Path {
    type Err = ParsePathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut path = s.trim();

        if path.is_empty() || matches!(path, "." | "/") {
            return Ok(Self::Root);
        }

        let mut segments = Vec::new();

        loop {
            let idx_sep_pair = path
                .as_bytes()
                .iter()
                .enumerate()
                .find_map(|(idx, byte)| Separator::from_byte(*byte).map(|sep| Ok((idx, sep))));

            let (sep_idx, sep) = match idx_sep_pair {
                Some(Ok(pair)) => pair,
                Some(Err(err)) => return Err(err),
                None => {
                    // if the string isnt empty, it's the final map key.
                    if !path.is_empty() {
                        segments.push(Segment::Map(path.to_owned()));
                    }

                    break;
                }
            };

            match sep {
                Separator::CloseBracket => return Err(ParsePathError::NoOpeningBracket(sep_idx)),
                Separator::Dot => {
                    let (current, remainder) = path.split_at(sep_idx);
                    path = remainder.get(1..).unwrap_or("");

                    // this can be empty if we're just coming from an index closing bracket, so
                    // skip if empty.
                    if !current.is_empty() {
                        segments.push(Segment::Map(current.to_owned()));
                    }
                }
                Separator::OpenBracket => {
                    // if there's any leading characters before the index bracket, it's a map key
                    // we need to add.
                    if let Some(leading) = path.get(..sep_idx)
                        && !leading.is_empty()
                    {
                        segments.push(Segment::Map(leading.to_owned()));
                    }

                    let closing_offset = path[sep_idx..]
                        .find(']')
                        .ok_or(ParsePathError::NoClosingBracket(sep_idx))?;

                    let closing_idx = sep_idx + closing_offset;

                    let index = path[(sep_idx + 1)..closing_idx]
                        .parse::<usize>()
                        .map_err(ParsePathError::InvalidIndex)?;

                    segments.push(Segment::Index(index));

                    path = path.get((closing_idx + 1)..).unwrap_or("");
                }
            }
        }

        if segments.is_empty() {
            Ok(Path::Root)
        } else {
            Ok(Path::Nested { segments })
        }
    }
}

/// A segment of a nested [`Path`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Segment {
    /// A segment in an integer indexed part of the data structure.
    Index(usize),
    /// A key in a map within the data structure.
    Map(String),
    /// An unknown segment. Formatted as `?` within a [`Path`].
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SegmentRef<'a> {
    Index(usize),
    Map(Cow<'a, str>),
    Unknown,
}

impl PartialEq<SegmentRef<'_>> for Segment {
    fn eq(&self, other: &SegmentRef<'_>) -> bool {
        self.as_ref() == *other
    }
}

impl Segment {
    fn as_ref(&self) -> SegmentRef<'_> {
        match self {
            Self::Index(idx) => SegmentRef::Index(*idx),
            Self::Map(key) => SegmentRef::Map(Cow::Borrowed(key.as_str())),
            Self::Unknown => SegmentRef::Unknown,
        }
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        fn cmp_idx_and_key(index: usize, key: &str) -> Ordering {
            // if the string key is a number, try and parse it and compare as a number.
            // if it errors out, dont worry about it, and treat the index as less than the
            // string key.
            match key.parse::<usize>().ok() {
                Some(parsed_idx) => index.cmp(&parsed_idx),
                _ => Ordering::Less,
            }
        }

        match (self, other) {
            // compare like types:
            (Self::Index(self_idx), Self::Index(other_idx)) => self_idx.cmp(other_idx),
            (Self::Map(self_key), Self::Map(other_key)) => self_key.cmp(other_key),
            (Self::Unknown, Self::Unknown) => Ordering::Equal,
            // Compare strings/indexes
            (Self::Index(self_idx), Self::Map(other_key)) => cmp_idx_and_key(*self_idx, other_key),
            (Self::Map(self_key), Self::Index(other_idx)) => {
                cmp_idx_and_key(*other_idx, self_key).reverse()
            }
            // then, fall back if only 1 is unknown.
            (_, Self::Unknown) => Ordering::Less,
            (Self::Unknown, _) => Ordering::Greater,
        }
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Index(idx) => write!(formatter, "[{idx}]"),
            Self::Map(key) => write!(formatter, "{key}"),
            Self::Unknown => write!(formatter, "?"),
        }
    }
}

impl From<String> for Segment {
    fn from(key: String) -> Self {
        Self::Map(key)
    }
}

impl From<&str> for Segment {
    fn from(key: &str) -> Self {
        Self::Map(key.to_owned())
    }
}

impl From<&String> for Segment {
    fn from(key: &String) -> Self {
        Self::Map(key.clone())
    }
}

impl From<Cow<'_, str>> for Segment {
    fn from(key: Cow<'_, str>) -> Self {
        Self::Map(key.into_owned())
    }
}

impl From<usize> for Segment {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl From<()> for Segment {
    fn from(_: ()) -> Self {
        Self::Unknown
    }
}

#[derive(Debug)]
pub struct PathIter<'a> {
    alive: std::ops::Range<usize>,
    segments: &'a [Segment],
}

impl PathIter<'_> {
    pub fn peek_next(&self) -> Option<&Segment> {
        // this prevents us from peeking forwards into segments that have already been yielded
        // via `DoubleEndedIterator::next_back`
        if self.alive.is_empty() {
            None
        } else {
            self.alive
                .start
                .checked_add(1)
                .and_then(|next_idx| self.segments.get(next_idx))
        }
    }

    pub fn peek_prev(&self) -> Option<&Segment> {
        // this prevents us from peeking backwards into segments that have already been yielded
        // via `Iterator::next`
        if self.alive.is_empty() {
            None
        } else {
            self.alive
                .end
                .checked_sub(1)
                .and_then(|prev_idx| self.segments.get(prev_idx))
        }
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = &'a Segment;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.segments.get(self.alive.start);
        self.alive.next();
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

#[test]
fn test_path_iter() {
    assert_eq!(Path::Root.iter().next(), None);
}

impl ExactSizeIterator for PathIter<'_> {
    fn len(&self) -> usize {
        self.alive.len()
    }
}

impl DoubleEndedIterator for PathIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ret = self.segments.get(self.alive.end);
        self.alive.next_back();
        ret
    }
}

impl Path {
    /// Returns the number of [`Segment`]s in the given [`Path`]. If `self` == [`Path::Root`],
    /// 0 is returned.
    pub fn len(&self) -> usize {
        match self {
            Self::Root => 0,
            Self::Nested { segments } => segments.len(),
        }
    }

    /// Returns whether or not [`Self`] is [`Self::Root`], or the segments container empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Root => true,
            Self::Nested { segments } => segments.is_empty(),
        }
    }

    #[allow(dead_code)]
    pub(super) fn eq_to_seg_track(&self, mut seg_track: &Track<'_>) -> bool {
        let mut self_iter = self.iter().rev();

        loop {
            seg_track = match (self_iter.next(), seg_track) {
                (None, track) => return *track == Track::Root,
                (Some(_), Track::Root) => return false,
                (Some(Segment::Unknown), Track::Unknown { parent }) => parent,
                (Some(Segment::Index(seg_idx)), Track::Sequence { parent, index }) => {
                    if seg_idx != index {
                        return false;
                    }

                    parent
                }
                (Some(Segment::Map(seg_key)), Track::Map { parent, key }) => {
                    if seg_key != key {
                        return false;
                    }

                    parent
                }
                _ => return false,
            }
        }
    }

    fn extend_from_track(&mut self, track: &Track<'_>) {
        if let Some(mut new_segs) = Self::from_seg_track_inner(None, track) {
            match self {
                Self::Nested { segments } => segments.append(&mut new_segs),
                _ => *self = Self::Nested { segments: new_segs },
            }
        }
    }

    fn from_seg_track_inner(
        child_segment: Option<Segment>,
        mut seg_track: &Track<'_>,
    ) -> Option<Vec<Segment>> {
        if *seg_track == Track::Root {
            match child_segment {
                Some(segment) => return Some(vec![segment]),
                _ => return None,
            }
        }

        let mut segments = child_segment.map(|seg| vec![seg]).unwrap_or_default();

        loop {
            seg_track = match seg_track {
                Track::Root => break,
                Track::Sequence { parent, index } => {
                    segments.push(Segment::Index(*index));
                    parent
                }
                Track::Map { parent, key } => {
                    segments.push(Segment::Map(key.clone()));
                    parent
                }
                Track::Unknown { parent } => {
                    segments.push(Segment::Unknown);
                    parent
                }
            };
        }

        // Since segments start at the child and iterate towards the root, we need to reverse
        // the segments so the root is at the front.
        segments.reverse();
        Some(segments)
    }

    /// internal helper for [`From`] impls. If the [`SegTrack`] we're converting from is owned,
    /// and a [`Map`] variant, we can avoid cloning the key string. The owned `From` impl peels
    /// off the first owned layer, then calls this with the parent to complete the path.
    ///
    /// The [`From<&SegTrack>`] impl calls this method directly, with `child_segment` = [`None`].
    fn from_seg_track(child_segment: Option<Segment>, seg_track: &Track<'_>) -> Self {
        match Self::from_seg_track_inner(child_segment, seg_track) {
            Some(segments) => Self::Nested { segments },
            None => Self::Root,
        }
    }

    /// Returns whether or not this [`Path`] is a [`Path::Root`] variant.
    pub fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    /// Returns an iterator over references to the [`Segment`]s.
    pub fn iter(&self) -> PathIter<'_> {
        match self {
            Self::Root => PathIter {
                alive: 0..0,
                segments: &[],
            },
            Self::Nested { segments } => PathIter {
                alive: 0..segments.len(),
                segments,
            },
        }
    }
}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a Segment;
    type IntoIter = PathIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Path {
    type Item = Segment;
    type IntoIter = std::vec::IntoIter<Segment>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Root => Vec::new().into_iter(),
            Self::Nested { segments } => segments.into_iter(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Track<'a> {
    Root,
    Sequence { parent: &'a Track<'a>, index: usize },
    Map { parent: &'a Track<'a>, key: String },
    Unknown { parent: &'a Track<'a> },
}

impl<'a> Track<'a> {
    pub fn add_map_child<S>(&'a self, key: S) -> Track<'a>
    where
        S: Into<String>,
    {
        Track::Map {
            parent: self,
            key: key.into(),
        }
    }

    pub fn add_seq_child(&'a self, index: usize) -> Track<'a> {
        Track::Sequence {
            parent: self,
            index,
        }
    }

    pub fn add_unknown_child(&'a self) -> Track<'a> {
        Track::Unknown { parent: self }
    }
}

impl From<&Track<'_>> for Path {
    fn from(track: &Track<'_>) -> Self {
        Self::from_seg_track(None, track)
    }
}

impl From<Track<'_>> for Path {
    fn from(track: Track<'_>) -> Self {
        let (segment, parent) = match track {
            Track::Root => return Self::Root,
            Track::Map { parent, key } => (Segment::Map(key), parent),
            Track::Sequence { parent, index } => (Segment::Index(index), parent),
            Track::Unknown { parent } => (Segment::Unknown, parent),
        };

        Self::from_seg_track(Some(segment), parent)
    }
}

impl From<std::borrow::Cow<'_, Track<'_>>> for Path {
    fn from(cow_seg_track: std::borrow::Cow<'_, Track<'_>>) -> Self {
        match cow_seg_track {
            Cow::Borrowed(borrowed) => borrowed.into(),
            Cow::Owned(owned) => owned.into(),
        }
    }
}
