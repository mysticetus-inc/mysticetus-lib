use super::cursor::StreamCursor;
use crate::tokens::{
    Location, PunctOrOp, Quote, QuoteFormat, QuotedModifier, Span, StringLiteral, Token,
};

/// Raw, inner tokenizer that can only be iterated forwards. No peeking/etc built in.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct InnerTokenizer<'src> {
    raw: &'src str,
    current: Location,
}

impl<'src> InnerTokenizer<'src> {
    #[inline]
    pub const fn new(sql: &'src str) -> Self {
        Self {
            raw: sql,
            current: Location {
                offset: 0,
                line: 0,
                column: 0,
            },
        }
    }

    pub(super) const fn new_from_location(location: Location, sub_sql: &'src str) -> Self {
        Self {
            raw: sub_sql,
            current: location,
        }
    }

    fn remainder(&self) -> Option<&'src str> {
        self.raw.get(self.current.offset..)
    }

    fn move_past_whitespace(&mut self) -> Option<StreamCursor<'src, '_>> {
        let rem = self.remainder()?;

        for ch in rem.chars() {
            if ch.is_whitespace() || ch.is_control() {
                self.current.advance_by(ch);
            } else {
                break;
            }
        }

        let remainder = self.remainder()?;

        Some(StreamCursor::new(remainder, &mut self.current))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum TokenizerError<'a> {
    #[error("invalid leading char found in sql: {0}")]
    InvalidChar(char),
    #[error("unbalanced delimiters, no '{expected}' found in '{rem}'")]
    Unbalanced { rem: &'a str, expected: char },
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("no closing quote ({0})")]
    NoClosingQuote(QuoteFormat),
}

struct RawStream<'src, 'tok> {
    iter: StreamCursor<'src, 'tok>,
}

impl<'src, 'tok> RawStream<'src, 'tok> {
    fn find_end_of_numeric_literal(&mut self) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
        fn is_valid_numeric_char(ch: char) -> bool {
            ch.is_numeric() || matches!(ch, 'E' | 'e' | '.')
        }

        self.iter.next_while(is_valid_numeric_char);

        Ok(self.iter.build_pair(Token::NumericLiteral))
    }

    fn find_end_of_unquoted_ident<F>(
        &mut self,
        to_token: F,
    ) -> Result<(Span, Token<'src>), TokenizerError<'src>>
    where
        F: FnOnce(&'src str) -> Token<'src>,
    {
        fn is_valid_ident_char(ch: char) -> bool {
            ch.is_alphanumeric() || matches!(ch, '_' | '-')
        }

        self.iter.next_while(is_valid_ident_char);

        Ok(self.iter.build_pair(to_token))
    }

    // inline comments are allowed to end the sql string, in the case
    // that there's no trailing newline, so this is infallible.
    fn move_past_inline_comment(&mut self) {
        self.iter.move_to_next('\n');
        self.iter.next();
    }

    // conversely, multiline comments must be closed with '*/', so this is fallible.
    fn move_past_multiline_comment(&mut self) -> Result<(), TokenizerError<'src>> {
        if self.iter.move_to_next_unescaped("*/") {
            self.iter.advance_bytes("*/".len());
            Ok(())
        } else {
            Err(TokenizerError::UnexpectedEof)
        }
    }

    fn find_closing_quoted_ident(&mut self) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
        if self.iter.move_to_next_unescaped('`') {
            self.iter.next();
            Ok(self.iter.build_pair(Token::QuotedIdentifier))
        } else {
            Err(TokenizerError::NoClosingQuote(QuoteFormat::Backticks))
        }
    }

    fn find_string_literal(
        &mut self,
        quote: Quote,
        modifier: Option<QuotedModifier>,
    ) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
        let leading = if modifier.is_some() {
            QuotedModifier::trim_from_start(self.iter.unadvanced_remainder())
        } else {
            self.iter.unadvanced_remainder()
        };

        let format = if leading.starts_with(quote.as_triple_str()) {
            if let Some(move_bytes) = quote
                .as_triple_str()
                .len()
                .checked_sub(self.iter.relative_offset())
            {
                self.iter.advance_bytes(move_bytes);
            }

            QuoteFormat::Triple(quote)
        } else {
            QuoteFormat::Single(quote)
        };

        if self.iter.move_to_next_unescaped(format.as_str()) {
            self.iter.advance_bytes(format.as_str().len());
            Ok(self.iter.build_pair(|raw_literal| {
                Token::StringLiteral(StringLiteral {
                    format,
                    modifier,
                    raw_literal,
                })
            }))
        } else {
            Err(TokenizerError::NoClosingQuote(format))
        }
    }

    fn find_closing_pair<F>(
        &mut self,
        opening: char,
        closing: char,
        to_token: F,
    ) -> Result<(Span, Token<'src>), TokenizerError<'src>>
    where
        F: FnOnce(&'src str) -> Token<'src>,
    {
        let mut balance = 1_u8;

        let mut escaped = false;

        while let Some((_, ch)) = self.iter.next() {
            if ch == '\\' {
                escaped = true;
                continue;
            }

            if ch == opening && !escaped {
                balance = balance.checked_add(1).expect("256 levels of nesting?");
            } else if ch == closing && !escaped {
                match balance.checked_sub(1) {
                    Some(sub) => balance = sub,
                    None => {
                        // move past the closing character.
                        self.iter.next();
                        return Ok(self.iter.build_pair(to_token));
                    }
                }
            } else {
                escaped = false;
            }
        }

        Err(TokenizerError::Unbalanced {
            rem: self.iter.remainder(),
            expected: closing,
        })
    }

    pub fn next_char(&mut self) -> Option<char> {
        self.iter.next().map(|(_, ch)| ch)
    }
}

impl<'a> Iterator for InnerTokenizer<'a> {
    type Item = Result<(Span, Token<'a>), TokenizerError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iter = self.move_past_whitespace()?;
        let (_, leading) = iter.next()?;

        let mut stream = RawStream { iter };

        match leading {
            // whitespace/control
            '\0'..=' ' => unreachable!(
                "leading chars should never be a whitespace char if 'move_past_whitespace' works \
                 properly (found {leading}, {})",
                leading as u8
            ),
            // A quoted identifier
            '`' => Some(stream.find_closing_quoted_ident()),
            // parens
            '(' => Some(stream.find_closing_pair('(', ')', Token::Parens)),
            // array indexing
            '[' => Some(stream.find_closing_pair('[', ']', Token::SquareBrackets)),
            // A numeric literal
            '0'..='9' => Some(stream.find_end_of_numeric_literal()),
            // A hint, or named query parameter
            '@' => Some(hint_or_query_param(&mut stream)),
            // StringLiteral
            '\"' => Some(stream.find_string_literal(Quote::Double, None)),
            '\'' => Some(stream.find_string_literal(Quote::Single, None)),
            '/' => match multiline_comment_or_div(&mut stream) {
                Ok(Some(pair)) => Some(Ok(pair)),
                // If none, we ran into a comment, and need to jump forwards.
                Ok(None) => self.next(),
                Err(err) => Some(Err(err)),
            },
            '-' => match inline_comment_or_neg(&mut stream) {
                Some(res) => Some(res),
                // If none, we ran into a comment, and need to jump forwards.
                None => self.next(),
            },
            // any alphabetic leading tokens, punctuation, etc
            _ => Some(parse_non_obvious(leading, &mut stream)),
        }
    }
}

fn parse_modified_literal_or_unquoted<'src>(
    stream: &mut RawStream<'src, '_>,
) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
    fn check_inner<'src2>(stream: &mut RawStream<'src2, '_>) -> Option<(Quote, QuotedModifier)> {
        let (modifier, quote) = QuotedModifier::from_leading(stream.iter.unadvanced_remainder())?;

        // this modifier is 2 chars, so we need to move 1 to get to before the quote char
        if modifier == QuotedModifier::RawBytes {
            stream.iter.next();
        }

        // move past 'quote'
        stream.iter.next();
        Some((quote, modifier))
    }

    match check_inner(stream) {
        Some((quote, modifier)) => stream.find_string_literal(quote, Some(modifier)),
        None => stream.find_end_of_unquoted_ident(Token::Unquoted),
    }
}

fn inline_comment_or_neg<'src>(
    stream: &mut RawStream<'src, '_>,
) -> Option<Result<(Span, Token<'src>), TokenizerError<'src>>> {
    if stream.iter.unadvanced_remainder().starts_with("--") {
        // if we dont find a newline, then the rest of the sql string is part of the comment.
        if !stream.iter.move_to_next('\n') {
            stream.iter.consume_remaining();
        }
        None
    } else if stream.iter.peek().is_some() {
        // cant have a trailing '-', so ensure there's more in the string.
        Some(Ok(stream
            .iter
            .build_pair(|_| Token::PunctOrOp(PunctOrOp::Neg))))
    } else {
        Some(Err(TokenizerError::UnexpectedEof))
    }
}

fn multiline_comment_or_div<'src>(
    stream: &mut RawStream<'src, '_>,
) -> Result<Option<(Span, Token<'src>)>, TokenizerError<'src>> {
    if stream.iter.unadvanced_remainder().starts_with("/*") {
        stream.move_past_multiline_comment()?;
        Ok(None)
    } else if stream.iter.unadvanced_remainder().len() == 1 {
        // cant have a trailing '/', so ensure there's more in the string.
        Err(TokenizerError::UnexpectedEof)
    } else {
        Ok(Some(
            stream.iter.build_pair(|_| Token::PunctOrOp(PunctOrOp::Div)),
        ))
    }
}

fn hint_or_query_param<'src>(
    stream: &mut RawStream<'src, '_>,
) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
    if stream.iter.next_if_eq('{') {
        // A hint
        stream.find_closing_pair('{', '}', Token::Hint)
    } else if stream.iter.peek().is_some() {
        // query parameter
        stream.find_end_of_unquoted_ident(Token::QueryParameter)
    } else {
        // EOF, can't have a trailing '@'
        Err(TokenizerError::UnexpectedEof)
    }
}

fn parse_non_obvious<'src>(
    leading: char,
    stream: &mut RawStream<'src, '_>,
) -> Result<(Span, Token<'src>), TokenizerError<'src>> {
    // alphabetic means something unquoted or a modified literal (i.e a raw string, r"string", etc).
    if leading.is_alphabetic() {
        return parse_modified_literal_or_unquoted(stream);
    }

    // identifiers can be lead by an underscore
    if leading == '_' {
        return stream.find_end_of_unquoted_ident(Token::Unquoted);
    }

    // otherwise it's some punctuating character
    if let Some(punct) = parse_punct_or_opt(leading, stream) {
        return Ok(stream.iter.build_pair(|_| Token::PunctOrOp(punct)));
    }

    // and if it's not one of those, it's not a char that can start off a new token
    Err(TokenizerError::InvalidChar(leading))
}

fn parse_punct_or_opt(leading: char, stream: &mut RawStream<'_, '_>) -> Option<PunctOrOp> {
    use PunctOrOp::*;

    const LT_OPTS: [(char, PunctOrOp); 3] = [('<', LeftShift), ('>', Eq), ('=', Lte)];

    const GT_OPTS: [(char, PunctOrOp); 2] = [('>', Eq), ('=', Gte)];

    const NOT_OPTS: [(char, PunctOrOp); 1] = [('=', NotEq)];

    const PIPE_OPTS: [(char, PunctOrOp); 1] = [('|', Concat)];

    match leading {
        '.' => Some(Dot),
        ',' => Some(Comma),
        '=' => Some(Eq),
        '<' => Some(stream.iter.first_next_if_eq_or(LT_OPTS, Lt)),
        '>' => Some(stream.iter.first_next_if_eq_or(GT_OPTS, Gt)),
        '-' => Some(Neg),
        '!' => Some(stream.iter.first_next_if_eq_or(NOT_OPTS, Not)),
        '~' => Some(Tilde),
        '*' => Some(Mul),
        ';' => Some(Semi),
        '|' => Some(stream.iter.first_next_if_eq_or(PIPE_OPTS, Or)),
        '/' => Some(Div),
        '+' => Some(Plus),
        '&' => Some(And),
        '^' => Some(Xor),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{InnerTokenizer, TokenizerError};
    use crate::tokens::{PunctOrOp, Quote, QuoteFormat, QuotedModifier, StringLiteral, Token};

    const TEST: &str =
        "SELECT This, That, AndTheOtherThing FROM TableA WHERE Num > 1.0 AND Str = /* comment 
                        */
                        \"This\" AND That = @qp AND Bytes = rb\"abcde\" AND Neg = -1e5 AND \
         EmptyTripleBlock = r'''''' AND TripleBlock = '''A Non Empty String With Escapes'''";

    const EXPECTING: &[Token<'static>] = &[
        Token::Unquoted("SELECT"),
        Token::Unquoted("This"),
        Token::PunctOrOp(PunctOrOp::Comma),
        Token::Unquoted("That"),
        Token::PunctOrOp(PunctOrOp::Comma),
        Token::Unquoted("AndTheOtherThing"),
        Token::Unquoted("FROM"),
        Token::Unquoted("TableA"),
        Token::Unquoted("WHERE"),
        Token::Unquoted("Num"),
        Token::PunctOrOp(PunctOrOp::Gt),
        Token::NumericLiteral("1.0"),
        Token::Unquoted("AND"),
        Token::Unquoted("Str"),
        Token::PunctOrOp(PunctOrOp::Eq),
        Token::StringLiteral(StringLiteral {
            modifier: None,
            format: QuoteFormat::Single(Quote::Double),
            raw_literal: "\"This\"",
        }),
        Token::Unquoted("AND"),
        Token::Unquoted("That"),
        Token::PunctOrOp(PunctOrOp::Eq),
        Token::QueryParameter("@qp"),
        Token::Unquoted("AND"),
        Token::Unquoted("Bytes"),
        Token::PunctOrOp(PunctOrOp::Eq),
        Token::StringLiteral(StringLiteral {
            modifier: Some(QuotedModifier::RawBytes),
            format: QuoteFormat::Single(Quote::Double),
            raw_literal: "rb\"abcde\"",
        }),
        Token::Unquoted("AND"),
        Token::Unquoted("Neg"),
        Token::PunctOrOp(PunctOrOp::Eq),
        Token::PunctOrOp(PunctOrOp::Neg),
        Token::NumericLiteral("1e5"),
        Token::Unquoted("AND"),
        Token::Unquoted("EmptyTripleBlock"),
        Token::PunctOrOp(PunctOrOp::Eq),
        Token::StringLiteral(StringLiteral {
            modifier: Some(QuotedModifier::Raw),
            format: QuoteFormat::Triple(Quote::Single),
            raw_literal: "r''''''",
        }),
        Token::Unquoted("AND"),
        Token::Unquoted("TripleBlock"),
        Token::PunctOrOp(PunctOrOp::Eq),
        Token::StringLiteral(StringLiteral {
            modifier: None,
            format: QuoteFormat::Triple(Quote::Single),
            raw_literal: "'''A Non Empty String With Escapes'''",
        }),
    ];

    #[test]
    fn tokenizer_basic_test() -> Result<(), TokenizerError<'static>> {
        assert!(InnerTokenizer::new(TEST).count() > 0);

        let mut expecting_iter = EXPECTING.iter().copied();

        for result in InnerTokenizer::new(TEST) {
            let (span, token) = result?;

            assert_eq!(token, &TEST[span.start.offset..][..span.len]);
            assert_eq!(expecting_iter.next().unwrap(), token);
        }

        assert_eq!(expecting_iter.next(), None);

        Ok(())
    }
}
