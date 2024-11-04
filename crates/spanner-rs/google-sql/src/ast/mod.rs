use std::borrow::Cow;
use std::fmt;

pub mod common;
pub mod ddl;
pub mod dml;
pub mod query;
pub mod reserved;

use self::common::punct::Comma;
use crate::Error;
use crate::tokens::{Span, Token, TokenizerKind};

#[derive(Debug, Clone, PartialEq)]
pub struct UnexpectedToken<'src> {
    found: Token<'src>,
    span: Span,
    expected: Option<Cow<'static, str>>,
}

impl<'src> UnexpectedToken<'src> {
    pub(crate) fn new(span: Span, found: Token<'src>) -> Self {
        Self {
            span,
            found,
            expected: None,
        }
    }

    pub(crate) fn new_expected<M>(span: Span, found: Token<'src>, expecting: M) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        Self::new(span, found).expected(expecting.into())
    }

    pub(crate) fn expected<M>(mut self, expecting: M) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        self.expected = Some(expecting.into());
        self
    }
}

impl std::error::Error for UnexpectedToken<'_> {}

impl fmt::Display for UnexpectedToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected token {:?} found", self.found)?;

        match self.expected.as_deref() {
            Some(expected) => write!(f, ", expected {expected}"),
            None => Ok(()),
        }
    }
}

/// for types that take many tokens to parse
pub(super) trait ParseTokens<'src>: Sized {
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>;

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        match Self::parse_tokens(&mut tokens.lookahead_iter()) {
            Ok(item) => Ok(Some(item)),
            Err(Error::UnexpectedToken(_) | Error::InvalidIdent(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

/// for types that consist of a single token (i.e a single identifier, operator, etc)
pub(super) trait FromToken<'src>: Sized {
    fn from_token(span: Span, token: Token<'src>) -> Result<Self, Error<'src>>;
}

impl<'src, P> ParseTokens<'src> for P
where
    P: FromToken<'src>,
{
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let (span, next) = tokens.next_or_eof()?;

        Self::from_token(span, next)
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        match tokens.peek()? {
            Some((span, token)) => match Self::from_token(span, token) {
                Ok(parsed) => {
                    tokens.next();
                    Ok(Some(parsed))
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}

impl<'src> ParseTokens<'src> for () {
    fn parse_tokens<T>(_tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        Ok(())
    }

    fn parse_optional<T>(_tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        Ok(Some(()))
    }
}

impl<'src, P> ParseTokens<'src> for Option<P>
where
    P: ParseTokens<'src>,
{
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        P::parse_optional(tokens)
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        match P::parse_optional(tokens)? {
            Some(item) => Ok(Some(Some(item))),
            None => Ok(None),
        }
    }
}

impl<'src, P> ParseTokens<'src> for Vec<P>
where
    P: ParseTokens<'src>,
{
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let mut dst = Vec::new();

        loop {
            match P::parse_optional(tokens)? {
                Some(elem) => dst.push(elem),
                None => break,
            }

            if Comma::parse_optional(tokens)?.is_none() {
                break;
            }
        }

        Ok(dst)
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let v = Self::parse_tokens(tokens)?;

        if v.is_empty() { Ok(None) } else { Ok(Some(v)) }
    }
}

impl<'src, P> ParseTokens<'src> for data_structures::non_empty_vec::NonEmptyVec<P>
where
    P: ParseTokens<'src>,
{
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let first = P::parse_tokens(tokens)?;

        if Comma::parse_optional(tokens)?.is_some() {
            let rem = Vec::<P>::parse_tokens(tokens)?;

            Ok(Self::from_parts(first, rem))
        } else {
            Ok(Self::from_parts(first, Vec::new()))
        }
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let first = match P::parse_optional(tokens)? {
            Some(first) => first,
            None => return Ok(None),
        };

        if Comma::parse_optional(tokens)?.is_some() {
            let rem = Vec::<P>::parse_tokens(tokens)?;

            Ok(Some(Self::from_parts(first, rem)))
        } else {
            Ok(Some(Self::from_parts(first, Vec::new())))
        }
    }
}

pub enum Statement<'src> {
    Query(query::QueryStatement<'src>),
    Ddl(ddl::DdlStatement<'src>),
    Dml(dml::DmlStatement<'src>),
}
