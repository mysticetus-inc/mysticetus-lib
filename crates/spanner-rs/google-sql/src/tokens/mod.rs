use std::fmt::{self, Write};
use std::hash::Hash;
mod pattern;
mod tokenizer;

pub use tokenizer::{Peek, Tokenizer, TokenizerError};

pub(crate) use self::tokenizer::LookaheadIter;
use crate::Error;
use crate::ast::reserved::{DataType, Keyword};

pub(crate) trait TokenizerKind<'src>:
    Iterator<Item = Result<(Span, Token<'src>), TokenizerError<'src>>>
{
    fn next_or_eof(&mut self) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
        self.next().ok_or(TokenizerError::UnexpectedEof).flatten()
    }

    fn peek(&mut self) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>>;

    fn peek_or_eof(&mut self) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
        self.peek()?.ok_or(TokenizerError::UnexpectedEof)
    }

    fn parse_from_token<T: crate::ast::FromToken<'src>>(
        &mut self,
    ) -> Result<Option<T>, Error<'src>> {
        match self.peek()? {
            Some((span, token)) => T::from_token(span, token).map(Some),
            None => Ok(None),
        }
    }

    fn next_if<F>(&mut self, predicate: F) -> Result<bool, TokenizerError<'src>>
    where
        F: FnOnce(Span, Token<'src>) -> bool,
    {
        match self.peek()? {
            Some((span, token)) if predicate(span, token) => {
                self.next();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn next_if_some<F, O>(&mut self, f: F) -> Result<Option<O>, TokenizerError<'src>>
    where
        F: FnOnce(Span, Token<'src>) -> Option<O>,
    {
        match self.peek()? {
            Some((span, token)) => {
                let opt = f(span, token);
                if opt.is_some() {
                    self.next();
                }
                Ok(opt)
            }
            None => Ok(None),
        }
    }

    fn lookahead_iter(&mut self) -> LookaheadIter<'_, 'src>;
}

impl<'src, T> TokenizerKind<'src> for &mut T
where
    T: TokenizerKind<'src>,
{
    fn peek(&mut self) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
        T::peek(self)
    }

    fn lookahead_iter(&mut self) -> LookaheadIter<'_, 'src> {
        T::lookahead_iter(self)
    }
}

impl<'src> TokenizerKind<'src> for Tokenizer<'src> {
    fn peek(&mut self) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
        self.peek_unpack()
    }

    fn lookahead_iter(&mut self) -> LookaheadIter<'_, 'src> {
        self.lookahead_iter()
    }
}

impl<'src> TokenizerKind<'src> for LookaheadIter<'_, 'src> {
    fn peek(&mut self) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
        self.peek_nth(0)
    }

    fn lookahead_iter(&mut self) -> LookaheadIter<'_, 'src> {
        self.reborrow()
    }
}

/// A single byte location within a SQL string
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    /// The offset from the start of the SQL string to this location, in bytes.
    pub offset: usize,
    /// The line number where this location starts (starts at 0)
    pub line: usize,
    /// The column index where this location starts (starts at 0)
    pub column: usize,
}

impl Location {
    /// convinience method to do 3 things that need to happen when moving to the next line:
    ///     - increments 'line' by 1
    ///     - resets 'column' to 0
    ///     - inrements 'offset' by 1, to move past the '\n'
    pub(crate) fn next_line(&mut self) {
        self.line += 1;
        self.offset += 1;
        self.column = 0;
    }

    pub(crate) fn advance_by(&mut self, ch: char) {
        if ch == '\n' {
            self.next_line();
        } else {
            self.offset += ch.len_utf8();
            self.column += 1;
        }
    }

    pub(crate) fn advance_by_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.advance_by(ch);
        }
    }
}

/// A [`Location`], covering a span of bytes within a SQL string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    /// The start byte,
    pub start: Location,
    /// the number of bytes covered by this span.
    pub len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct StringLiteral<'a> {
    modifier: Option<QuotedModifier>,
    format: QuoteFormat,
    raw_literal: &'a str,
}

impl<'a> StringLiteral<'a> {
    pub fn contents(&self) -> &'a str {
        let no_mods = if self.modifier.is_some() {
            QuotedModifier::trim_from_start(self.raw_literal)
        } else {
            self.raw_literal
        };

        no_mods
            .trim_start_matches(self.format.as_str())
            .trim_end_matches(self.format.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Token<'a> {
    /// An unquoted sequence of characters. Could be a keyword, identifier, numeric literal, etc.
    Unquoted(&'a str),
    /// A quoted identifier, quoted with <code>`</code>.
    QuotedIdentifier(&'a str),
    /// A string literal, that may or may not be modified. Quoted by either <code>\'</code> or
    /// <code>\"</code>. Inner string includes the quotes.
    StringLiteral(StringLiteral<'a>),
    /// A numeric literal
    NumericLiteral(&'a str),
    /// A Google-SQL hint, in the format '@{ ... }'
    Hint(&'a str),
    /// A named query parameter, prefixed with '@'.
    QueryParameter(&'a str),
    /// A sub-expression/etc wrapped in parenthases. includes the parens.
    Parens(&'a str),
    /// An indexing expression within square brackets. includes the brackets.
    SquareBrackets(&'a str),
    /// A punctuation or operator character(s). Could be field access, the start of a nested
    /// typedef, or an operator.
    PunctOrOp(PunctOrOp),
}

impl PartialEq<PunctOrOp> for Token<'_> {
    fn eq(&self, other: &PunctOrOp) -> bool {
        match self.as_punct_or_opt() {
            Some(punct) => punct.eq(other),
            None => false,
        }
    }
}

impl PartialEq<StringLiteral<'_>> for Token<'_> {
    fn eq(&self, other: &StringLiteral<'_>) -> bool {
        match self.as_string_literal() {
            Some(lit) => lit.eq(other),
            None => false,
        }
    }
}
macro_rules! impl_as_fns {
    ($($name:ident($variant:ident) -> $out:ty),* $(,)?) => {
        $(
            #[inline]
            pub fn $name(&self) -> Option<$out> {
                match *self {
                    Self::$variant(value) => Some(value),
                    _ => None,
                }
            }
        )*
    };
}

fn build_nested_tokenizer<'src>(
    nested_expr: &'src str,
    opening: char,
    closing: char,
    mut location: Location,
) -> Tokenizer<'src> {
    let no_leading = nested_expr.trim_start_matches(opening);

    let delta_len = nested_expr.len() - no_leading.len();

    debug_assert!(delta_len > 0, "no leading characters stripped?");

    location.offset -= delta_len;
    location.column -= delta_len / opening.len_utf8();

    let no_wrapping = no_leading.trim_end_matches(closing);

    Tokenizer::new_from_location(location, no_wrapping)
}

impl<'a> Token<'a> {
    impl_as_fns! {
        as_unquoted(Unquoted) -> &'a str,
        as_quoted_identifier(QuotedIdentifier) -> &'a str,
        as_string_literal(StringLiteral) -> StringLiteral<'a>,
        as_numeric_literal(NumericLiteral) -> &'a str,
        as_hint(Hint) -> &'a str,
        as_query_parameter(QueryParameter) -> &'a str,
        as_parens(Parens) -> &'a str,
        as_square_brackets(SquareBrackets) -> &'a str,
        as_punct_or_opt(PunctOrOp) -> PunctOrOp,
    }

    pub fn as_keyword(&self) -> Option<Keyword> {
        self.as_unquoted().and_then(Keyword::from_str)
    }

    pub fn as_data_type(&self) -> Option<DataType> {
        self.as_unquoted().and_then(DataType::from_str)
    }

    pub fn is_keyword(&self, kw: Keyword) -> bool {
        match self.as_unquoted() {
            Some(unquoted) => unquoted.eq_ignore_ascii_case(kw.as_str()),
            None => false,
        }
    }

    pub fn build_parens_parser(&self, location: Location) -> Option<Tokenizer<'a>> {
        let sub = self.as_parens()?;

        Some(build_nested_tokenizer(sub, '(', ')', location))
    }

    pub fn build_square_brackets_parser(&self, location: Location) -> Option<Tokenizer<'a>> {
        let sub = self.as_square_brackets()?;

        Some(build_nested_tokenizer(sub, '(', ')', location))
    }

    fn str_eq_inner(&self, s: &str) -> bool {
        match *self {
            Self::Unquoted(uq) => uq == s,
            Self::QueryParameter(q) => q == s,
            Self::QuotedIdentifier(ident) => ident == s,
            Self::StringLiteral(lit) => lit.raw_literal == s,
            Self::NumericLiteral(num) => num == s,
            Self::Hint(hint) => hint == s,
            Self::Parens(p) => p == s,
            Self::SquareBrackets(sb) => sb == s,
            Self::PunctOrOp(p) => p.eq_str(s),
        }
    }
}

impl<S: AsRef<str>> PartialEq<S> for Token<'_> {
    fn eq(&self, other: &S) -> bool {
        self.str_eq_inner(other.as_ref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuotedModifier {
    // a raw string, specifed with 'r"..."'
    Raw,
    // a byte string, specifed with 'b"..."'
    Bytes,
    // a raw byte string, specifed with 'rb"..."'
    RawBytes,
}

impl QuotedModifier {
    pub fn trim_from_start(s: &str) -> &str {
        s.trim_start_matches(|ch: char| matches!(ch.to_ascii_lowercase(), 'r' | 'b'))
    }

    pub fn from_leading(s: &str) -> Option<(Self, Quote)> {
        let mut chars = s.chars().map(|ch| ch.to_ascii_lowercase());

        let first = chars.next()?;
        let second = chars.next()?;

        let (modifier, quote_char) = match (first, second) {
            ('r', 'b') | ('b', 'r') => (QuotedModifier::RawBytes, chars.next()?),
            ('r', _) => (QuotedModifier::Raw, second),
            ('b', _) => (QuotedModifier::Bytes, second),
            _ => return None,
        };

        let quote = Quote::from_char(quote_char)?;
        Some((modifier, quote))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Quote {
    Single = b'\'',
    Double = b'\"',
}

impl Quote {
    fn as_char(&self) -> char {
        *self as u8 as char
    }

    // these 2 functions are to make use with QuoteFormat nicer,
    // without needing to do on-the-fly character encoding.
    // if there was more than 2 options, maybe, but this is simpler.
    fn as_single_str(&self) -> &'static str {
        match self {
            Self::Single => "\'",
            Self::Double => "\"",
        }
    }

    fn as_triple_str(&self) -> &'static str {
        match self {
            Self::Single => "'''",
            Self::Double => "\"\"\"",
        }
    }

    fn from_leading(s: &str) -> Option<Self> {
        s.chars().next().and_then(Self::from_char)
    }

    pub(super) fn from_char(ch: char) -> Option<Self> {
        match ch {
            '\'' => Some(Self::Single),
            '\"' => Some(Self::Double),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuoteFormat {
    /// An item quoted in backticks
    Backticks,
    /// An item surrounded by a single set of quotes
    Single(Quote),
    /// An item surrounded by triple quotes set of quotes (multiline usually)
    Triple(Quote),
}

impl QuoteFormat {
    fn len(&self) -> usize {
        match self {
            Self::Backticks | Self::Single(_) => 1,
            Self::Triple(_) => 3,
        }
    }
    fn from_leading(s: &str) -> Option<Self> {
        let quote = match s.chars().next()? {
            '\'' => Quote::Single,
            '\"' => Quote::Double,
            '`' => return Some(Self::Backticks),
            _ => return None,
        };

        if s.starts_with(quote.as_triple_str()) {
            Some(Self::Triple(quote))
        } else {
            Some(Self::Single(quote))
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Backticks => "`",
            Self::Single(quote) => quote.as_single_str(),
            Self::Triple(quote) => quote.as_triple_str(),
        }
    }

    fn prefixes(&self, s: &str) -> bool {
        s.starts_with(self.as_str())
    }
}

impl fmt::Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(*self as u8 as char)
    }
}

impl fmt::Display for QuoteFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            QuoteFormat::Backticks => f.write_char('`'),
            QuoteFormat::Single(single) => single.fmt(f),
            QuoteFormat::Triple(triple) => {
                triple.fmt(f)?;
                triple.fmt(f)?;
                triple.fmt(f)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PunctOrOp {
    Dot,
    Comma,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    LeftShift,
    RightShift,
    Or,
    Concat,
    Neg,
    Not,
    Tilde,
    Mul,
    Semi,
    Div,
    Plus,
    And,
    Xor,
}

impl PunctOrOp {
    pub fn eq_str(&self, s: &str) -> bool {
        use PunctOrOp::*;

        match self {
            Dot => "." == s,
            Comma => "," == s,
            Eq => "=" == s,
            NotEq => "!=" == s || "<>" == s,
            Lt => "<" == s,
            Lte => "<=" == s,
            Gt => ">" == s,
            Gte => ">=" == s,
            LeftShift => "<<" == s,
            RightShift => ">>" == s,
            Or => "|" == s,
            Concat => "||" == s,
            Neg => "-" == s,
            Not => "!" == s,
            Tilde => "~" == s,
            Mul => "*" == s,
            Semi => ";" == s,
            Div => "/" == s,
            Plus => "+" == s,
            And => "&" == s,
            Xor => "^" == s,
        }
    }
}
