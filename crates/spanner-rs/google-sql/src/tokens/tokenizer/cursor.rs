use std::str::pattern::Pattern;

use super::{Location, Span, Token};

/// Inner iterator over characters that keeps track of the location in the sql stream,
/// and adds helper functions. Intentionally in a private module to keep inner details
/// isolated.
#[derive(Debug)]
pub(super) struct StreamCursor<'src, 'tok> {
    remainder: &'src str,
    start: Location,
    current: &'tok mut Location,
    iter: std::iter::Peekable<std::str::CharIndices<'src>>,
}

impl<'src, 'tok> StreamCursor<'src, 'tok> {
    pub(super) fn reset(&mut self) {
        *self.current = self.start;
        self.iter = self.remainder.char_indices().peekable();
    }

    pub(super) fn is_escaped(&self) -> bool {
        let Some(prev) = self.relative_offset().checked_sub(1) else {
            return false;
        };

        self.remainder.is_char_boundary(prev) && self.remainder[prev..].chars().next() == Some('\\')
    }

    pub(super) fn unadvanced_remainder(&self) -> &'src str {
        self.remainder
    }

    pub(super) fn relative_offset(&self) -> usize {
        self.current.offset - self.start.offset
    }

    pub(super) fn consume_remaining(&mut self) {
        while self.next().is_some() {}
    }

    pub(super) fn remainder(&self) -> &'src str {
        &self.remainder[self.relative_offset()..]
    }

    pub(super) fn advance_bytes(&mut self, bytes: usize) {
        assert!(
            self.remainder().is_char_boundary(bytes),
            "can't offset to a non-char boundary"
        );

        let mut count = 0;

        loop {
            match self.next() {
                Some((_, ch)) => {
                    count += ch.len_utf8();
                    if bytes <= count {
                        break;
                    }
                }
                None => break,
            }
        }
    }

    pub(super) fn move_to_next<P>(&mut self, pat: P) -> bool
    where
        P: Pattern,
    {
        match self.remainder().find(pat) {
            Some(offset) => {
                self.advance_bytes(offset);
                true
            }
            None => false,
        }
    }

    pub(super) fn move_to_next_unescaped<P>(&mut self, pat: P) -> bool
    where
        P: Pattern + Copy,
    {
        while self.move_to_next(pat) {
            if self.is_escaped() {
                self.next();
            } else {
                return true;
            }
        }

        false
    }

    pub(super) fn peek(&mut self) -> Option<char> {
        self.iter.peek().map(|(_, ch)| *ch)
    }

    pub fn next_if<F>(&mut self, pred: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        if self.peek().is_some_and(pred) {
            self.next();
            true
        } else {
            false
        }
    }

    pub fn next_if_eq(&mut self, target: char) -> bool {
        self.next_if(|ch| ch == target)
    }

    pub fn next_while<F>(&mut self, pred: F) -> usize
    where
        F: FnMut(char) -> bool + Copy,
    {
        let mut count = 0;

        while self.next_if(pred) {
            count += 1;
        }

        count
    }

    pub fn first_next_if_eq<I, T>(&mut self, options: I) -> Option<T>
    where
        I: IntoIterator<Item = (char, T)>,
    {
        for (target, option) in options {
            if self.next_if_eq(target) {
                return Some(option);
            }
        }

        None
    }

    pub fn first_next_if_eq_or<I, T>(&mut self, options: I, default: T) -> T
    where
        I: IntoIterator<Item = (char, T)>,
    {
        self.first_next_if_eq(options.into_iter())
            .unwrap_or(default)
    }

    pub(super) fn advance_to_offset(&mut self, offset: usize) {
        // if we're already here, neat
        if self.relative_offset() == offset {
            return;
        }

        // if the offset is past the end of the string, return (panic in debug)
        if self.remainder.len() < offset {
            #[cfg(debug_assertions)]
            {
                panic!(
                    "offset of {offset} is past the end of the string (len = {})",
                    self.remainder.len()
                );
            }
            #[cfg(not(debug_assertions))]
            {
                return;
            }
        }

        assert!(
            self.remainder.is_char_boundary(offset),
            "cant offset to a non-char boundary"
        );

        // reset if we're already past the offset for some reason.
        if self.relative_offset() > offset {
            self.reset();
        }

        self.current
            .advance_by_str(&self.remainder[self.relative_offset()..offset]);

        self.iter = self.remainder[offset..].char_indices().peekable();
    }

    fn advance_by(&mut self, n_chars: usize) {
        for _ in 0..n_chars {
            self.next();
        }
    }

    pub(super) fn new(remainder: &'src str, current: &'tok mut Location) -> Self {
        Self {
            start: *current,
            current,
            iter: remainder.char_indices().peekable(),
            remainder,
        }
    }

    pub(super) fn build_pair<F>(&self, to_token: F) -> (Span, Token<'src>)
    where
        F: FnOnce(&'src str) -> Token<'src>,
    {
        (
            self.build_span(),
            to_token(&self.remainder[..self.relative_offset()]),
        )
    }

    pub(super) fn build_span(&self) -> Span {
        let len = self.current.offset - self.start.offset;

        Span {
            start: self.start,
            len,
        }
    }
}

impl<'src> Iterator for StreamCursor<'src, '_> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let (index, ch) = self.iter.next()?;
        self.current.advance_by(ch);
        Some((index, ch))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
