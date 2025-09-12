use std::borrow::Cow;
use std::fmt;
use std::slice::{Iter, IterMut};

const SEP_STR: &str = "/";
const SEP_CHAR: char = '/';

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Path<'a> {
    Ref(Vec<Cow<'a, str>>),
    Owned(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnedPath {
    segments: Vec<String>,
}

pub trait RtDbPath: Clone {
    fn complete_base_url(&self, base: &mut String);
}

impl RtDbPath for Path<'_> {
    fn complete_base_url(&self, base: &mut String) {
        self.complete_base_url(base)
    }
}

impl RtDbPath for OwnedPath {
    fn complete_base_url(&self, base: &mut String) {
        self.complete_base_url(base)
    }
}

impl OwnedPath {
    pub(crate) fn from_path(path: Path<'_>) -> Self {
        let segments = match path {
            Path::Owned(segments) => segments,
            Path::Ref(segments) => segments
                .into_iter()
                .map(Cow::into_owned)
                .collect::<Vec<String>>(),
        };

        Self { segments }
    }

    pub(crate) fn n_chars(&self) -> usize {
        let segments = self.n_segments();

        if segments == 0 {
            return 0;
        }

        let chars_without_seps: usize = self.segments.iter().map(|s| s.len()).sum();

        chars_without_seps + segments - 1
    }

    pub fn complete_base_url(&self, base: &mut String) {
        if !base.ends_with(SEP_CHAR) {
            base.push(SEP_CHAR);
        }

        let n_chars = self.n_chars() + 5; // .json extension adds 5 more chars

        if n_chars > base.capacity() - base.len() {
            base.reserve(n_chars - (base.capacity() - base.len()));
        }

        let last_idx = self.n_segments() - 1;
        for (idx, seg) in self.segments.iter().enumerate() {
            base.push_str(&*seg);

            if idx != last_idx {
                base.push(SEP_CHAR);
            }
        }

        base.push_str(".json");
    }

    pub(crate) fn into_path(self) -> Path<'static> {
        Path::Owned(self.segments)
    }

    pub(crate) fn pop(&mut self) -> Option<String> {
        self.segments.pop()
    }

    pub fn n_segments(&self) -> usize {
        self.segments.len()
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }

    pub(crate) fn push(&mut self, child: String) {
        if child.is_empty() {
            return;
        }

        if !child.contains(SEP_CHAR) {
            self.segments.push(child);
            return;
        }

        let split_segment_iter = child
            .split_terminator(SEP_CHAR)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned());

        self.segments.extend(split_segment_iter);
    }
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let final_idx = self.n_segments() - 1;
        for (idx, seg) in self.iter().enumerate() {
            formatter.write_str(&*seg)?;

            if idx != final_idx {
                formatter.write_str(SEP_STR)?;
            }
        }

        Ok(())
    }
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(string: &'a str) -> Self {
        Self::from_str(string)
    }
}

impl<'a> From<&'a String> for Path<'a> {
    fn from(string: &'a String) -> Self {
        Self::from_str(string.as_str())
    }
}

impl From<String> for Path<'_> {
    fn from(string: String) -> Self {
        Self::from_string(string)
    }
}

impl<'a> From<Cow<'a, str>> for Path<'a> {
    fn from(cow: Cow<'a, str>) -> Self {
        match cow {
            Cow::Borrowed(borrowed) => Self::from_str(borrowed),
            Cow::Owned(owned) => Self::from_string(owned),
        }
    }
}

impl<'a> Path<'a> {
    pub fn from_str(string: &'a str) -> Self {
        let mut new = Self::with_capacity(1);
        new.push_str(string);
        new
    }

    pub fn into_owned(self) -> OwnedPath {
        OwnedPath::from_path(self)
    }

    pub fn from_string(string: String) -> Self {
        let mut new = Self::with_capacity(1);
        new.push_string(string);
        new
    }

    pub fn clear(&mut self) {
        match self {
            Self::Ref(seg) => seg.clear(),
            Self::Owned(seg) => seg.clear(),
        }
    }

    pub const fn new() -> Self {
        Self::Ref(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::Ref(Vec::with_capacity(capacity))
    }

    pub fn pop(&mut self) -> Option<Cow<'_, str>> {
        match self {
            Self::Ref(seg) => seg.pop(),
            Self::Owned(seg) => seg.pop().map(Cow::Owned),
        }
    }

    pub fn n_segments(&self) -> usize {
        match self {
            Self::Ref(seg) => seg.len(),
            Self::Owned(seg) => seg.len(),
        }
    }

    pub(crate) fn n_chars(&self) -> usize {
        let segments = self.n_segments();

        if segments == 0 {
            return 0;
        }

        let chars_without_seps: usize = match self {
            Self::Ref(seg) => seg.iter().map(|s| s.len()).sum(),
            Self::Owned(seg) => seg.iter().map(|s| s.len()).sum(),
        };

        chars_without_seps + segments - 1
    }

    pub fn complete_base_url(&self, base: &mut String) {
        if !base.ends_with(SEP_CHAR) {
            base.push(SEP_CHAR);
        }

        let n_chars = self.n_chars() + 5; // .json extension adds 5 more chars

        if n_chars > base.capacity() - base.len() {
            base.reserve(n_chars - (base.capacity() - base.len()));
        }

        let last_idx = self.n_segments() - 1;
        for (idx, seg) in self.iter().enumerate() {
            base.push_str(&*seg);

            if idx != last_idx {
                base.push(SEP_CHAR);
            }
        }

        base.push_str(".json");
    }

    pub fn push_display<D>(&mut self, display: D)
    where
        D: fmt::Display,
    {
        self.push_string(display.to_string());
    }

    pub fn push_str(&mut self, segment: &'a str) {
        if segment.is_empty() {
            return;
        }

        if !segment.contains(SEP_CHAR) {
            match self {
                Self::Ref(seg) => seg.push(Cow::Borrowed(segment)),
                Self::Owned(seg) => seg.push(segment.to_owned()),
            }
            return;
        }

        let split_segment_iter = segment.split_terminator(SEP_CHAR).filter(|s| !s.is_empty());

        match self {
            Self::Ref(seg) => seg.extend(split_segment_iter.map(Cow::Borrowed)),
            Self::Owned(seg) => seg.extend(split_segment_iter.map(|s| s.to_owned())),
        }
    }

    pub fn push_string(&mut self, string: String) {
        if string.is_empty() {
            return;
        }

        if !string.contains(SEP_CHAR) {
            match self {
                Self::Ref(seg) => seg.push(Cow::Owned(string)),
                Self::Owned(seg) => seg.push(string),
            }
            return;
        }

        let split_segment_iter = string.split_terminator(SEP_CHAR).filter(|s| !s.is_empty());

        match self {
            Self::Ref(seg) => seg.extend(split_segment_iter.map(|s| Cow::Owned(s.to_owned()))),
            Self::Owned(seg) => seg.extend(split_segment_iter.map(Into::into)),
        }
    }

    fn get_inner_mut(&mut self) -> &mut Vec<String> {
        let vec = match self {
            Self::Owned(owned) => std::mem::take(owned),
            Self::Ref(refer) => std::mem::take(refer)
                .into_iter()
                .map(Cow::into_owned)
                .collect(),
        };

        *self = Self::Owned(vec);

        if let Self::Owned(owned) = self {
            return owned;
        }

        unreachable!()
    }

    pub fn iter(&self) -> PathIter<'_> {
        let inner = match self {
            Self::Ref(refer) => InnerPathIter::Ref(refer.iter()),
            Self::Owned(owned) => InnerPathIter::Owned(owned.iter()),
        };

        PathIter(inner)
    }

    pub fn iter_mut(&mut self) -> PathIterMut<'_> {
        PathIterMut(self.get_inner_mut().iter_mut())
    }
}

enum InnerPathIter<'a> {
    Ref(Iter<'a, Cow<'a, str>>),
    Owned(Iter<'a, String>),
}

pub struct PathIter<'a>(InnerPathIter<'a>);

impl<'a> Iterator for PathIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            InnerPathIter::Ref(i) => i.next().map(|s| &**s),
            InnerPathIter::Owned(i) => i.next().map(|s| s.as_str()),
        }
    }
}

impl<'a> DoubleEndedIterator for PathIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            InnerPathIter::Ref(i) => i.next_back().map(|s| &**s),
            InnerPathIter::Owned(i) => i.next_back().map(|s| s.as_str()),
        }
    }
}

impl<'a> ExactSizeIterator for PathIter<'a> {
    fn len(&self) -> usize {
        match &self.0 {
            InnerPathIter::Ref(i) => i.len(),
            InnerPathIter::Owned(i) => i.len(),
        }
    }
}

pub struct PathIterMut<'a>(IterMut<'a, String>);

impl<'a> Iterator for PathIterMut<'a> {
    type Item = &'a mut String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> DoubleEndedIterator for PathIterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for PathIterMut<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut segments = Path::with_capacity(5);
        segments.push_str("nested/like/a/mfer");
        segments.push_display(5);

        assert_eq!(Some("5".into()), segments.pop());
        assert_eq!(Some("mfer".into()), segments.pop());
        assert_eq!(Some("a".into()), segments.pop());
        assert_eq!(Some("like".into()), segments.pop());
        assert_eq!(Some("nested".into()), segments.pop());
    }
}
