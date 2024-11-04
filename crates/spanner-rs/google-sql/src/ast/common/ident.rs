use data_structures::non_empty_vec::NonEmptyVec;

use crate::Error;
use crate::ast::reserved::Keyword;
use crate::ast::{FromToken, ParseTokens, UnexpectedToken};
use crate::tokens::{PunctOrOp, Span, Token, TokenizerKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ident<'src> {
    QueryParam(&'src str),
    Quoted(&'src str),
    Unquoted(&'src str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidIdent<'src> {
    #[error("unquoted identifier cannot be keyword: {0:?}")]
    UnquotedKeyword(Keyword),
    #[error("identifier cannot be empty")]
    EmptyIdent,
    #[error("invalid identifier '{ident}', {reason}")]
    InvalidIdent {
        ident: &'src str,
        reason: &'static str,
    },
}

fn validate_ident(ident: &str) -> Result<(), InvalidIdent<'_>> {
    // strip a leading query parameter symbol only once
    let ident = if ident.starts_with('@') {
        &ident[1..]
    } else {
        ident
    };

    // we know it'll also end with backticks, since the tokenizer will only return valid
    // quoted sequences.
    if ident.starts_with('`') {
        validate_quoted(ident)
    } else {
        validate_unquoted(ident)
    }
}

fn validate_quoted(ident: &str) -> Result<(), InvalidIdent<'_>> {
    if ident
        .trim_start_matches('`')
        .trim_end_matches('`')
        .is_empty()
    {
        Err(InvalidIdent::EmptyIdent)
    } else {
        Ok(())
    }
}

fn validate_unquoted(ident: &str) -> Result<(), InvalidIdent<'_>> {
    if ident.is_empty() {
        return Err(InvalidIdent::EmptyIdent);
    }

    if let Some(kw) = Keyword::from_str(ident) {
        return Err(InvalidIdent::UnquotedKeyword(kw));
    }

    if ident.starts_with(char::is_numeric) {
        return Err(InvalidIdent::InvalidIdent {
            ident,
            reason: "cannot start with a number",
        });
    }

    if ident.starts_with('-') || ident.ends_with('-') {
        return Err(InvalidIdent::InvalidIdent {
            ident,
            reason: "cannot start or end with '-'",
        });
    }

    Ok(())
}

impl<'src> FromToken<'src> for Ident<'src> {
    fn from_token(span: Span, token: Token<'src>) -> Result<Self, Error<'src>> {
        match token {
            Token::Unquoted(unquoted) => {
                validate_unquoted(unquoted)?;
                Ok(Ident::Unquoted(unquoted))
            }
            Token::QueryParameter(qp) => {
                validate_ident(qp)?;
                Ok(Ident::QueryParam(qp))
            }
            Token::QuotedIdentifier(quoted) => {
                validate_quoted(quoted)?;
                Ok(Ident::Quoted(quoted))
            }
            _ => Err(UnexpectedToken::new_expected(span, token, "a valid identifier").into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<'src> {
    segments: NonEmptyVec<Ident<'src>>,
}

fn parse_remaining_path<'src, T>(
    dst: &mut Vec<Ident<'src>>,
    tokens: &mut T,
) -> Result<(), Error<'src>>
where
    T: TokenizerKind<'src>,
{
    while tokens.next_if(|_, token| token.as_punct_or_opt() == Some(PunctOrOp::Dot))? {
        let next = Ident::parse_tokens(tokens)?;
        dst.push(next);
    }

    Ok(())
}

impl<'src> ParseTokens<'src> for Path<'src> {
    fn parse_tokens<T>(tokens: &mut T) -> Result<Self, Error<'src>>
    where
        T: crate::tokens::TokenizerKind<'src>,
    {
        let leading = Ident::parse_tokens(tokens)?;

        let mut segments = Vec::new();
        parse_remaining_path(&mut segments, tokens)?;

        Ok(Self {
            segments: NonEmptyVec::from_parts(leading, segments),
        })
    }

    fn parse_optional<T>(tokens: &mut T) -> Result<Option<Self>, Error<'src>>
    where
        T: TokenizerKind<'src>,
    {
        let leading = match Ident::parse_optional(tokens)? {
            Some(leading) => leading,
            None => return Ok(None),
        };

        let mut segments = Vec::new();
        parse_remaining_path(&mut segments, tokens)?;

        Ok(Some(Self {
            segments: NonEmptyVec::from_parts(leading, segments),
        }))
    }
}
