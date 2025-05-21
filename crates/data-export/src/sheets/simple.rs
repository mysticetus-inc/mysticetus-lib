use super::BaseSheet;
use super::private::SealedIntoSheet;
use crate::{Cell, CellContent};

pub struct SimpleSheet<'xlsx> {
    inner: BaseSheet<'xlsx>,
}

impl<'xlsx> SimpleSheet<'xlsx> {
    pub(crate) fn new(inner: BaseSheet<'xlsx>) -> Self {
        Self { inner }
    }

    pub fn incr_row(&mut self) {
        self.inner.incr_row();
    }

    pub fn cell(&mut self) -> Cell<'_> {
        self.inner.cell()
    }

    pub fn write_row<R>(&mut self, row: R) -> crate::Result<()>
    where
        R: IntoIterator,
        R::Item: CellContent,
    {
        let mut cell = self.cell();
        for (idx, value) in row.into_iter().enumerate() {
            value.format(cell.set_col(idx as u16))?;
        }
        self.incr_row();
        Ok(())
    }

    pub fn write_rows<R>(&mut self, rows: R) -> crate::Result<()>
    where
        R: IntoIterator,
        R::Item: IntoIterator,
        <R::Item as IntoIterator>::Item: CellContent,
    {
        for row_iter in rows {
            self.write_row(row_iter)?;
        }
        Ok(())
    }
}

impl SealedIntoSheet for SimpleSheet<'static> {
    #[inline]
    fn into_sheet(self) -> Box<rust_xlsxwriter::Worksheet> {
        self.inner.into_sheet()
    }
}
