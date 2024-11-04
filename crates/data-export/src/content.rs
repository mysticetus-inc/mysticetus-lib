use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;

use crate::cell::Cell;
use crate::error::Error;

/// Trait for formatting a type into a cell in an excel sheet.
///
/// Meant to strongly resemble [`fmt::Debug`] and [`fmt::Display`], in that [`Cell`] provides
/// similar helper methods as [`fmt::Formatter`].
pub trait CellContent {
    /// Formats [`Self`] into the [`Cell`].
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error>;

    /// Formats [`Self`] into the [`Cell`], in a type specific 'pretty' format. Default
    /// implementation defers to the required [`CellContent::format`] method.
    fn format_pretty(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        self.format(cell)
    }
}

impl CellContent for str {
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        cell.write_str(self)
    }
}

impl CellContent for String {
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        cell.write_str(self.as_str())
    }
}

impl<T> CellContent for Cow<'_, T>
where
    T: CellContent + ToOwned + ?Sized,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format(&self, cell)
    }

    fn format_pretty(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format_pretty(&self, cell)
    }
}

pub trait ColHeader: CellContent + Ord + Hash + Clone + Sized {}

impl<T> ColHeader for T where T: CellContent + Ord + Hash + Clone + Sized {}

impl<T> CellContent for &T
where
    T: CellContent + ?Sized,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format(self, cell)
    }

    fn format_pretty(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format_pretty(self, cell)
    }
}

impl<T> CellContent for Option<T>
where
    T: CellContent,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        match self.as_ref() {
            Some(inner) => inner.format(cell),
            None => cell.write_blank(),
        }
    }

    fn format_pretty(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        match self.as_ref() {
            Some(inner) => inner.format_pretty(cell),
            None => cell.write_blank(),
        }
    }
}

impl<T> CellContent for Box<T>
where
    T: CellContent + ?Sized,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format(&*self, cell)
    }

    fn format_pretty(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format_pretty(&*self, cell)
    }
}

impl<T> CellContent for std::sync::Arc<T>
where
    T: CellContent + ?Sized,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format(&*self, cell)
    }

    fn format_pretty(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        T::format_pretty(&*self, cell)
    }
}

/// Wrapper type that implemnts [`CellContent`] for any inner type that implements
/// [`fmt::Display`].
pub struct Display<'a, T: ?Sized>(pub &'a T);

impl<T> CellContent for Display<'_, T>
where
    T: fmt::Display + ?Sized,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        cell.write_display(self.0)
    }
}

/// Wrapper type that implemnts [`CellContent`] for any inner type that implements
/// [`fmt::Debug`].
pub struct Debug<'a, T: ?Sized>(pub &'a T);

impl<T> CellContent for Debug<'_, T>
where
    T: fmt::Debug + ?Sized,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        cell.write_debug(self.0)
    }
}

/// A combo wrapper type that merges the behavior of the [`Debug`] + [`Display`] wrapper types.
///
/// If the inner type implements both [`fmt::Debug`] + [`fmt::Display`], this implements
/// [`CellContent`]. Depending on the [`CellContent`] method called, this uses either one of the
/// [`fmt`] impls. The mapping is:
///
/// - [`CellContent::format`] -> [`fmt::Debug`]
/// - [`CellContent::format_pretty`] -> [`fmt::Display`]
pub struct DebugDisplay<'a, T: ?Sized>(pub &'a T);

impl<T> CellContent for DebugDisplay<'_, T>
where
    T: fmt::Debug + fmt::Display + ?Sized,
{
    fn format(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        cell.write_debug(self.0)
    }

    fn format_pretty(&self, cell: &mut Cell<'_>) -> Result<(), Error> {
        cell.write_display(self.0)
    }
}
