use std::ops::DerefMut;
use std::{fmt, io};

use genco::prelude::Rust;
use genco::quote_in;
use genco::tokens::FormatInto;

pub struct DocFormatter<'a, B: DerefMut<Target = String> = &'a mut String> {
    parent: std::str::SplitTerminator<'a, char>,
    truncate_at: usize,
    buffer: B,
    is_done: bool,
    next_word: Option<&'a str>,
}

const INDENT: &str = "    ";
const DOC_BEGIN: &str = "/// ";
const MOD_DOC_BEGIN: &str = "//! ";

const MAX_LINE_LEN: usize = 100;
// const DOC_LINE_LEN_LIMIT: usize = MAX_LINE_LEN - DOC_BEGIN.len();

impl<'a, B> DocFormatter<'a, B>
where
    B: DerefMut<Target = String>,
{
    fn new_inner(being_str: &str, parent: &'a str, indent_level: usize, mut buffer: B) -> Self {
        buffer.clear();

        for _ in 0..indent_level {
            buffer.push_str(INDENT);
        }

        buffer.push_str(being_str);

        Self {
            parent: parent.split_terminator(' '),
            truncate_at: buffer.len(),
            next_word: None,
            is_done: false,
            buffer,
        }
    }

    pub fn new_doc(parent: &'a str, indent_level: usize, buffer: B) -> Self {
        Self::new_inner(DOC_BEGIN, parent, indent_level, buffer)
    }

    pub fn new_module_doc(parent: &'a str, indent_level: usize, buffer: B) -> Self {
        Self::new_inner(MOD_DOC_BEGIN, parent, indent_level, buffer)
    }

    pub fn io_write_into<W: io::Write>(mut self, writer: &mut W) -> io::Result<()> {
        while let Some(line) = self.next_line() {
            writeln!(writer, "{line}")?;
        }

        Ok(())
    }

    pub fn fmt_write_into<W: fmt::Write>(mut self, writer: &mut W) -> fmt::Result {
        while let Some(line) = self.next_line() {
            writeln!(writer, "{line}")?;
        }

        Ok(())
    }

    fn next_line(&mut self) -> Option<&str> {
        loop {
            if let Some(next_word) = self.next_word.take() {
                self.buffer.truncate(self.truncate_at);
                self.buffer.push_str(next_word);
            }

            let next = match self.parent.next() {
                Some(next) => next,
                None if self.is_done => return None,
                None => {
                    self.is_done = true;
                    return Some(self.buffer.as_str());
                }
            };

            if self.buffer.len() + next.len() >= MAX_LINE_LEN {
                self.next_word = Some(next);

                return Some(self.buffer.as_str());
            } else {
                if !self.buffer.ends_with(' ') {
                    self.buffer.push(' ');
                }

                self.buffer.push_str(next);
            }
        }
    }
}

impl<B: DerefMut<Target = String>> FormatInto<Rust> for DocFormatter<'_, B> {
    fn format_into(mut self, tokens: &mut genco::Tokens<Rust>) {
        while let Some(line) = self.next_line() {
            quote_in! { *tokens => $line $['\r'] }
        }
    }
}
