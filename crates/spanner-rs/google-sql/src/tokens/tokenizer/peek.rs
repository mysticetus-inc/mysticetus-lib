use std::collections::VecDeque;

use super::inner::{InnerTokenizer, TokenizerError};
use crate::tokens::{Span, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct PeekableTokenizer<'src> {
    inner: InnerTokenizer<'src>,
    peeked: VecDeque<(Span, Token<'src>)>,
    /// error 'fuse', meant to indicate we threw an error and this type can't proceed further.
    error: Option<TokenizerError<'src>>,
}

#[derive(Debug, PartialEq)]
pub struct Peek<'p, 'src> {
    parent: &'p mut PeekableTokenizer<'src>,
    pub token: Token<'src>,
    pub span: Span,
}

impl<'p, 'src> Peek<'p, 'src> {
    pub fn consume(self) -> (Span, Token<'src>) {
        self.parent
            .next()
            .transpose()
            .expect("should be ok if this type exists");

        (self.span, self.token)
    }
}

impl<'src> PeekableTokenizer<'src> {
    pub(super) fn new(inner: InnerTokenizer<'src>) -> Self {
        Self {
            inner,
            // start with a bit of initial capacity, this is likely enough to make sure
            // we'll never need to resize.
            peeked: VecDeque::with_capacity(32),
            error: None,
        }
    }

    pub fn continuous_buffered(&mut self) -> Result<&[(Span, Token<'src>)], TokenizerError<'src>> {
        self.check_error()?;
        Ok(self.peeked.make_contiguous())
    }

    pub fn buffered(&self) -> Result<&VecDeque<(Span, Token<'src>)>, TokenizerError<'src>> {
        self.check_error()?;
        Ok(&self.peeked)
    }

    pub fn consume_n(&mut self, n: usize) -> Result<(), TokenizerError<'src>> {
        for _ in 0..n {
            if self.next_inner()?.is_none() {
                break;
            }
        }

        Ok(())
    }

    pub fn peek_nth(
        &mut self,
        index: usize,
    ) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
        self.check_error()?;
        self.fill_peek_to_n(index)?;
        Ok(self.peeked.get(index).copied())
    }

    #[inline]
    fn check_error(&self) -> Result<(), TokenizerError<'src>> {
        if let Some(ref error) = self.error {
            Err(error.clone())
        } else {
            Ok(())
        }
    }

    #[inline]
    fn handle_item<T>(
        &mut self,
        item: Result<T, TokenizerError<'src>>,
    ) -> Result<T, TokenizerError<'src>> {
        if let Err(ref error) = item {
            self.error = Some(error.clone());
        }

        item
    }

    fn next_inner(&mut self) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
        self.check_error()?;

        if let Some(peek) = self.peeked.pop_front() {
            return Ok(Some(peek));
        }

        match self.inner.next() {
            Some(item) => self.handle_item(item).map(Some),
            None => Ok(None),
        }
    }

    pub fn peek_n<const N: usize>(
        &mut self,
    ) -> Result<Option<[(Span, Token<'src>); N]>, TokenizerError<'src>> {
        self.fill_peek_to_n(N)?;

        Ok(std::array::try_from_fn(|index| {
            self.peeked.get(index).copied()
        }))
    }

    fn fill_peek_to_n(&mut self, n: usize) -> Result<bool, TokenizerError<'src>> {
        if self.peeked.len() >= n {
            self.check_error()?;
            return Ok(true);
        }

        while self.peeked.len() < n {
            if !self.peek_next_inner()? {
                break;
            }
        }

        Ok(self.peeked.len() >= n)
    }

    fn peek_next_inner(&mut self) -> Result<bool, TokenizerError<'src>> {
        self.check_error()?;

        match self.inner.next() {
            Some(item) => {
                let token = self.handle_item(item)?;
                self.peeked.push_back(token);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn peek(&mut self) -> Result<Option<Peek<'_, 'src>>, TokenizerError<'src>> {
        if let Some((span, token)) = self.peeked.front().copied() {
            // next_inner checks internally, but if we're pulling from here, we need to check now
            self.check_error()?;
            return Ok(Some(Peek {
                parent: self,
                span,
                token,
            }));
        }

        let next = self.next_inner();

        match self.handle_item(next)? {
            None => Ok(None),
            Some(pair) => {
                self.peeked.push_back(pair);
                let (span, token) = self.peeked.back().copied().unwrap();

                Ok(Some(Peek {
                    parent: self,
                    span,
                    token,
                }))
            }
        }
    }
}

impl<'src> Iterator for PeekableTokenizer<'src> {
    type Item = Result<(Span, Token<'src>), TokenizerError<'src>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner().transpose()
    }
}
