//! Tokenizer for Google Standard Sql.
//!
//! Implementation uses several layers of abstraction, mainly to make AST parsing a bit nicer.
//!
//! Layers are described below, from the lowest level of abstraction to the highest.
//!
//! [`cursor::StreamCursor`]: iterator that starts at the beginning of a token with the goal of
//! locating the end of a single token. Doesn't try and derive any meaning from the token while
//! searching.
//!
//! [`inner::InnerTokenizer`]: Inner tokenizer that uses [`StreamCursor`] in each call to
//! [`Iterator::next`] to find and parse the next [`Token`]. Doesn't support any lookahead,
//! reverse iteration, etc.
//!
//! [`peek::PeekableTokenizer`]: Wrapper around [`InnerTokenizer`] that buffers tokens in a
//! deque to allow for lookahead/peeking.
//!
//! Lastly, [`Tokenizer`], a thin wrapper around [`PeekableTokenizer`] which implements the real
//! helper functions for AST parsing. Main interface in the AST parsing traits,
//! [`FromToken`]/[`ParseTokens`]
//!
//! [`StreamCursor`]: cursor::StreamCursor
//! [`InnerTokenizer`]: inner::InnerTokenizer
//! [`PeekableTokenizer`]: peek::PeekableTokenizer
//! [`FromToken`]: crate::ast::FromToken
//! [`ParseTokens`]: crate::ast::ParseTokens

mod cursor;
mod inner;
mod peek;

use data_structures::maybe_owned_mut::MaybeOwnedMut;
pub use inner::TokenizerError;
pub use peek::Peek;

use super::{Location, PunctOrOp, Span, Token};
use crate::Error;
use crate::ast::reserved::Keyword;

/// Peekable iterator over <code>([`Span`], [`Token`])</code> generated from a raw Google-SQL
/// string.
///
/// Essentially a wrapper around [`peek::PeekableTokenizer`] (which itself is a wrapper over the
/// [`inner::InnerTokenizer`]). This provides all the utility functions needed for AST parsing,
/// in a somewhat efficient way.
#[derive(Debug, Clone, PartialEq)]
pub struct Tokenizer<'src> {
    raw: &'src str,
    inner: peek::PeekableTokenizer<'src>,
}

/// Iterator adapter that causes the tokens to continually peek forwards when iterating, only
/// consuming when [`LookaheadIter::consume`] is called.
#[derive(Debug, PartialEq)]
pub struct LookaheadIter<'b, 'src> {
    next_peek_index: MaybeOwnedMut<'b, usize>,
    parent: &'b mut Tokenizer<'src>,
}

impl<'src> Iterator for LookaheadIter<'_, 'src> {
    type Item = Result<(Span, Token<'src>), TokenizerError<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next_peek_index.as_mut();
        let res = self.parent.inner.peek_nth(*next).transpose();
        *next += 1;
        res
    }
}

impl<'b, 'src> LookaheadIter<'b, 'src> {
    pub(super) fn reborrow(&mut self) -> LookaheadIter<'_, 'src> {
        LookaheadIter {
            next_peek_index: MaybeOwnedMut::MutRef(self.next_peek_index.as_mut()),
            parent: self.parent,
        }
    }

    pub(crate) fn peek_nth(
        &mut self,
        index: usize,
    ) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
        self.parent.inner.peek_nth(index)
    }

    fn new(parent: &'b mut Tokenizer<'src>) -> Self {
        Self {
            next_peek_index: MaybeOwnedMut::Owned(0),
            parent,
        }
    }

    pub(crate) fn consume(self) -> Result<usize, TokenizerError<'src>> {
        let count = self.next_peek_index.saturating_sub(1);
        if count > 0 {
            self.parent.inner.consume_n(count)?;
        }
        Ok(count)
    }
}

impl<'src> Tokenizer<'src> {
    #[inline]
    pub fn new(sql: &'src str) -> Self {
        Self {
            raw: sql,
            inner: peek::PeekableTokenizer::new(inner::InnerTokenizer::new(sql)),
        }
    }

    pub fn lookahead_iter(&mut self) -> LookaheadIter<'_, 'src> {
        LookaheadIter::new(self)
    }

    pub fn new_from_location(location: Location, sub_sql: &'src str) -> Self {
        Self {
            raw: sub_sql,
            inner: peek::PeekableTokenizer::new(inner::InnerTokenizer::new_from_location(
                location, sub_sql,
            )),
        }
    }

    #[inline]
    pub(crate) fn peek(&mut self) -> Result<Option<peek::Peek<'_, 'src>>, TokenizerError<'src>> {
        self.inner.peek()
    }

    #[inline]
    pub(crate) fn peek_unpack(
        &mut self,
    ) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
        match self.inner.peek()? {
            Some(peek) => Ok(Some((peek.span, peek.token))),
            None => Ok(None),
        }
    }

    /// convinience wrapper that calls ParseTokens::parse_tokens for 'P', but also
    /// does checks to unwrap parenthases and build an inner tokenizer if that's the case.
    pub(super) fn parse<P>(&mut self) -> Result<P, Error<'src>>
    where
        P: crate::ast::ParseTokens<'src>,
    {
        if let Some(mut nested) =
            self.next_if_some(|span, token| token.build_parens_parser(span.start))?
        {
            let out = P::parse_tokens(&mut nested)?;
            // TODO: check to ensure that nested is completely exhausted of non-comment tokens
            Ok(out)
        } else {
            P::parse_tokens(self)
        }
    }

    #[inline]
    pub fn input(&self) -> &'src str {
        self.raw
    }

    pub fn next_if_keyword(&mut self) -> Result<Option<Keyword>, TokenizerError<'src>> {
        self.next_if_some(|_, token| token.as_keyword())
    }

    pub fn next_or_eof(&mut self) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
        self.next().ok_or(TokenizerError::UnexpectedEof)?
    }

    pub fn next_if_punct(&mut self, punct: PunctOrOp) -> Result<bool, TokenizerError<'src>> {
        match self.peek()? {
            Some(peek) if peek.token.as_punct_or_opt() == Some(punct) => {
                peek.consume();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn next_if_some<F, O>(&mut self, map: F) -> Result<Option<O>, TokenizerError<'src>>
    where
        F: FnOnce(Span, Token<'src>) -> Option<O>,
    {
        let mut ret = None;

        self.next_if(|span, token| {
            ret = map(span, token);
            ret.is_some()
        })?;

        Ok(ret)
    }

    pub fn next_if<F>(
        &mut self,
        predicate: F,
    ) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>>
    where
        F: FnOnce(Span, Token<'src>) -> bool,
    {
        match self.inner.peek()? {
            Some(peek) => {
                if predicate(peek.span, peek.token) {
                    Ok(Some(peek.consume()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl<'src> Iterator for Tokenizer<'src> {
    type Item = Result<(Span, Token<'src>), TokenizerError<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
