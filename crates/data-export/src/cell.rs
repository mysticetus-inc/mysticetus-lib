use std::fmt::{self, Write};

use crate::CellContent;
use crate::col_fmt::{ColSums, Integer, count_float_digits};
use crate::error::Error;

/// A cell in an Excel worksheet. Used to write a value into the cell.
pub struct Cell<'sheet> {
    pub(crate) sheet: &'sheet mut rust_xlsxwriter::Worksheet,
    pub(crate) col: u16,
    pub(crate) row: &'sheet mut u32,
    pub(crate) buf: &'sheet mut String,
    // Since there's no way to inspect the current state of a [`Format`], we need to track bold
    // on it's own, since it factors into sizing calculations.
    pub(crate) bold: bool,
    pub(crate) col_width_sums: &'sheet mut Vec<ColSums>,
    pub(crate) format: Option<rust_xlsxwriter::Format>,
}

/// helper macro for getting access to a format reference when calling worksheet 'write_XXX' methods
macro_rules! format_ref {
    ($self:expr) => {{
        match $self.format {
            Some(ref f) => f,
            None => $self.format.insert(rust_xlsxwriter::Format::new()),
        }
    }};
}

impl<'sheet> Cell<'sheet> {
    pub fn format<I>(&mut self, value: &I) -> Result<(), Error>
    where
        I: CellContent,
    {
        value.format(self)
    }

    pub fn format_pretty<I>(&mut self, value: &I) -> Result<(), Error>
    where
        I: CellContent,
    {
        value.format_pretty(self)
    }

    pub fn write_args(&mut self, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.buf.clear();
        fmt::write(&mut self.buf, args)?;
        self.write_buffer()
    }

    pub(crate) fn set_col(&mut self, col: u16) -> &mut Self {
        self.col = col;
        self
    }

    pub(crate) fn incr_row(&mut self) -> &mut Self {
        *self.row += 1;
        self
    }

    pub fn write_display<T>(&mut self, display: T) -> Result<(), Error>
    where
        T: fmt::Display,
    {
        self.write_args(format_args!("{}", display))
    }

    pub fn take_format(&mut self) -> rust_xlsxwriter::Format {
        self.format
            .take()
            .unwrap_or_else(rust_xlsxwriter::Format::new)
    }

    pub fn set_bold(&mut self) -> &mut Self {
        self.bold = true;
        self.format = Some(self.take_format().set_bold());
        self
    }

    pub fn write_debug<T>(&mut self, debug: T) -> Result<(), Error>
    where
        T: fmt::Debug,
    {
        self.write_args(format_args!("{:?}", debug))
    }

    pub fn write_blank(&mut self) -> Result<(), Error> {
        self.add_col_term(0);
        self.sheet
            .write_blank(*self.row, self.col, format_ref!(self))?;
        Ok(())
    }

    pub fn clear_format(&mut self) -> &mut Self {
        self.format = None;
        self.bold = false;
        self
    }

    fn font_size(&self) -> f64 {
        let base_size = if let Some(_format) = self.format.as_ref() {
            // TODO:
            crate::DEFAULT_FONT_SIZE
        } else {
            crate::DEFAULT_FONT_SIZE
        };

        if self.bold {
            1.25 * base_size
        } else {
            base_size
        }
    }

    pub fn write_str<S>(&mut self, s: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        let string = s.as_ref();

        self.add_col_term(string.len());

        self.sheet
            .write_string(*self.row, self.col, string, format_ref!(self))?;

        Ok(())
    }

    fn write_num_inner(&mut self, float: f64, digits: usize) -> Result<(), Error> {
        self.add_col_term(digits);
        self.sheet
            .write_number(*self.row, self.col, float, format_ref!(self))?;
        Ok(())
    }

    pub fn write_float<D>(&mut self, float: f64, limit_digits: D) -> Result<(), Error>
    where
        D: Into<Option<u32>>,
    {
        match float.classify() {
            std::num::FpCategory::Nan => return self.write_str("NaN"),
            std::num::FpCategory::Infinite if float.is_sign_positive() => {
                return self.write_str("Inf");
            }
            std::num::FpCategory::Infinite => return self.write_str("-Inf"),
            _ => (),
        }

        let num = if let Some(digits) = limit_digits.into() as Option<u32> {
            let mult = 10_u64.pow(digits) as f64;

            (float * mult).trunc() / mult
        } else {
            float
        };

        self.write_num_inner(num, count_float_digits(num))
    }

    pub fn write_boolean(&mut self, b: bool) -> Result<(), Error> {
        self.add_col_term(if b { 4 } else { 5 });
        self.sheet
            .write_boolean(*self.row, self.col, b, format_ref!(self))?;
        Ok(())
    }

    pub fn write_boolean_as_string(&mut self, b: bool) -> Result<(), Error> {
        self.write_str(if b { "true" } else { "false" })
    }

    pub fn build_context(&mut self) -> WriteContext<'_, 'sheet> {
        self.buf.clear();
        WriteContext { cell: self }
    }

    pub fn write_integer<I>(&mut self, int: I) -> Result<(), Error>
    where
        I: Integer,
    {
        self.write_num_inner(int.as_float(), int.num_fmt_chars())
    }

    fn add_col_term(&mut self, n_chars: usize) -> &mut Self {
        while self.col as usize >= self.col_width_sums.len() {
            self.col_width_sums.push(ColSums::default());
        }

        let width = (self.font_size() * n_chars as f64) / crate::DEFAULT_FONT_SIZE;

        self.col_width_sums[self.col as usize].add_term(width);
        self
    }

    fn write_buffer(&mut self) -> Result<(), Error> {
        self.add_col_term(self.buf.len());

        self.sheet
            .write_string(*self.row, self.col, &*self.buf, format_ref!(self))?;

        Ok(())
    }
}

impl Write for Cell<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.write_str(s) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        match self.write_args(args) {
            Ok(_) => Ok(()),
            _ => Err(fmt::Error),
        }
    }
}

#[must_use = "must call [`WriteContext::commit`] to write out the buffer to [`Cell`]"]
pub struct WriteContext<'ctx, 'sheet> {
    cell: &'ctx mut Cell<'sheet>,
}

impl WriteContext<'_, '_> {
    pub fn push_str<S>(&mut self, string: S)
    where
        S: AsRef<str>,
    {
        self.cell.buf.push_str(string.as_ref());
    }

    pub fn get_buffer(&mut self) -> &mut String {
        self.cell.buf
    }

    pub fn peek_buffer(&self) -> &str {
        self.cell.buf.as_str()
    }

    pub fn push_args(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        Write::write_fmt(&mut self.cell.buf, args)
    }

    pub fn push_display<D>(&mut self, display: D) -> fmt::Result
    where
        D: fmt::Display,
    {
        write!(&mut self.cell.buf, "{}", display)
    }

    pub fn push_debug<D>(&mut self, debug: D) -> fmt::Result
    where
        D: fmt::Debug,
    {
        write!(&mut self.cell.buf, "{:?}", debug)
    }

    pub fn commit(self) -> Result<(), Error> {
        self.cell.write_buffer()
    }
}

impl Write for WriteContext<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.push_args(args)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.cell.buf.push(c);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color {
    pub red: u8,
    pub blue: u8,
    pub green: u8,
}

#[macro_export]
macro_rules! rgb {
    ($red:literal, $green:literal, $blue:literal) => {{
        $crate::cell::Color {
            red: $red,
            green: $green,
            blue: $blue,
        }
    }};
}

impl Color {
    pub const WHITE: Self = rgb!(255, 255, 255);
    pub const BLACK: Self = rgb!(0, 0, 0);
    pub const RED: Self = rgb!(255, 0, 0);
    pub const GREEN: Self = rgb!(0, 255, 0);
    pub const BLUE: Self = rgb!(0, 0, 255);

    #[inline]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
    ///
    /// ```
    /// # use data_export::cell::Color;
    /// let white = Color {
    ///     red: 255,
    ///     blue: 255,
    ///     green: 255,
    /// };
    /// assert_eq!(white.to_hex_value(), 0xFFFFFF);
    ///
    /// let red = Color {
    ///     red: 255,
    ///     blue: 0,
    ///     green: 0,
    /// };
    /// assert_eq!(red.to_hex_value(), 0xFF0000);
    ///
    /// let black = Color {
    ///     red: 0,
    ///     blue: 0,
    ///     green: 0,
    /// };
    /// assert_eq!(black.to_hex_value(), 0x000000);
    /// ```
    pub const fn to_hex_value(self) -> u32 {
        ((self.red as u32) << 16) + ((self.blue as u32) << 8) + (self.green as u32)
    }
}
