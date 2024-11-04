use self::ident::Ident;
use super::{ParseTokens, UnexpectedToken};
use crate::Error;
use crate::tokens::TokenizerKind;

pub mod expression;
pub mod ident;
pub mod operator;
pub mod punct;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsAlias<'src, T> {
    pub item: T,
    pub alias: Option<Ident<'src>>,
}

impl<'src, P> AsAlias<'src, P> {
    fn parse_alias<T>(item: P, tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        match tokens
            .peek()?
            .and_then(|(_span, token)| token.as_unquoted())
        {
            Some(unquoted) if unquoted.eq_ignore_ascii_case("as") => {
                // move past 'AS'
                tokens.next().transpose()?;

                let alias = Ident::parse_tokens(tokens)?;

                Ok(Self {
                    item,
                    alias: Some(alias),
                })
            }
            _ => Ok(Self { item, alias: None }),
        }
    }
}

impl<'src, P> ParseTokens<'src> for AsAlias<'src, P>
where
    P: ParseTokens<'src>,
{
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let item = P::parse_tokens(tokens)?;
        AsAlias::parse_alias(item, tokens)
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        match P::parse_optional(tokens)? {
            Some(item) => AsAlias::parse_alias(item, tokens).map(Some),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cast<A, B> {
    pub from: A,
    pub to: B,
}

impl<A, B> Cast<A, B> {
    fn parse_remainder<'src, T>(from: A, tokens: &mut T) -> Result<Self, Error<'src>>
    where
        B: ParseTokens<'src>,
        T: TokenizerKind<'src>,
    {
        let (span, token) = tokens.next_or_eof()?;

        if token
            .as_unquoted()
            .is_some_and(|unquoted| unquoted.eq_ignore_ascii_case("as"))
        {
            let to = B::parse_tokens(tokens)?;

            Ok(Self { from, to })
        } else {
            Err(UnexpectedToken::new(span, token)
                .expected("the 'AS' keyword")
                .into())
        }
    }
}

impl<'src, A, B> ParseTokens<'src> for Cast<A, B>
where
    A: ParseTokens<'src>,
    B: ParseTokens<'src>,
{
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let from = A::parse_tokens(tokens)?;
        Self::parse_remainder(from, tokens)
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        match A::parse_optional(tokens)? {
            Some(from) => Self::parse_remainder(from, tokens).map(Some),
            None => Ok(None),
        }
    }
}
