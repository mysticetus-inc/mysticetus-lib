use crate::Error;
use crate::ast::{ParseTokens, UnexpectedToken};
use crate::tokens::{PunctOrOp, Token, TokenizerKind};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Comma;

impl<'src> ParseTokens<'src> for Comma {
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: crate::tokens::TokenizerKind<'src>,
    {
        let (span, token) = tokens.next_or_eof()?;

        match token.as_punct_or_opt() {
            Some(PunctOrOp::Comma) => Ok(Self),
            _ => Err(UnexpectedToken::new(span, token).expected("a comma").into()),
        }
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: crate::tokens::TokenizerKind<'src>,
    {
        match tokens.peek()? {
            Some((_, Token::PunctOrOp(PunctOrOp::Comma))) => {
                tokens.next();
                Ok(Some(Self))
            }
            _ => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeParenthasized<T> {
    Parens(T),
    NoParens(T),
}

fn parse_parenthasized<'src, P, T>(tokens: &mut T) -> Option<Result<P, Error<'src>>>
where
    P: ParseTokens<'src>,
    T: TokenizerKind<'src>,
{
    match tokens.next_if_some(|span, token| token.build_parens_parser(span.start)) {
        Ok(Some(mut inner_tokenizer)) => {
            let res = P::parse_tokens(&mut inner_tokenizer);

            // TODO: ensure the nested tokenizer is out of important items (i.e non-comments).

            Some(res)
        }
        Ok(None) => None,
        Err(err) => Some(Err(err.into())),
    }
}

impl<'src, P> ParseTokens<'src> for MaybeParenthasized<P>
where
    P: ParseTokens<'src>,
{
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: crate::tokens::TokenizerKind<'src>,
    {
        match parse_parenthasized(tokens) {
            Some(res) => res.map(MaybeParenthasized::Parens),
            None => P::parse_tokens(tokens).map(MaybeParenthasized::NoParens),
        }
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        match parse_parenthasized::<P, T>(tokens) {
            Some(res) => res.map(|item| Some(MaybeParenthasized::Parens(item))),
            None => P::parse_optional(tokens).map(|item| item.map(MaybeParenthasized::NoParens)),
        }
    }
}
