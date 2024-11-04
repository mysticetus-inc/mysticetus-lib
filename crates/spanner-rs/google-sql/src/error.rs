use crate::ast::UnexpectedToken;
use crate::ast::common::ident::InvalidIdent;
use crate::tokens::TokenizerError;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error<'src> {
    #[error(transparent)]
    Tokenizer(TokenizerError<'src>),
    #[error(transparent)]
    UnexpectedToken(UnexpectedToken<'src>),
    #[error(transparent)]
    InvalidIdent(InvalidIdent<'src>),
}

impl<'src> From<UnexpectedToken<'src>> for Error<'src> {
    fn from(value: UnexpectedToken<'src>) -> Self {
        Self::UnexpectedToken(value)
    }
}

impl<'src> From<TokenizerError<'src>> for Error<'src> {
    fn from(value: TokenizerError<'src>) -> Self {
        Self::Tokenizer(value)
    }
}

impl<'src> From<InvalidIdent<'src>> for Error<'src> {
    fn from(value: InvalidIdent<'src>) -> Self {
        Self::InvalidIdent(value)
    }
}
