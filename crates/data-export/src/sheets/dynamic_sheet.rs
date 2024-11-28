use std::collections::HashMap;

use super::BaseSheet;
use crate::Cell;
use crate::content::{CellContent, ColHeader};
use crate::error::Error;

/// An excel sheet with dynamic columns that are appended as encountered.
#[must_use = "must call 'finalize' or columns will not be written."]
pub struct DynamicSheet<'xlsx, C> {
    base: BaseSheet<'xlsx>,
    cols_written: bool,
    cols: HashMap<C, u16>,
}

impl<C> super::private::SealedIntoSheet for DynamicSheet<'static, C> {
    fn into_sheet(self) -> rust_xlsxwriter::Worksheet {
        self.base.into_sheet()
    }
}

#[inline]
fn col_index<C: ColHeader>(cols: &mut HashMap<C, u16>, col: &C) -> u16 {
    let len = cols.len();
    let (_, index) = cols
        .raw_entry_mut()
        .from_key(col)
        .or_insert_with(|| (C::clone(col), len as u16));

    *index
}

#[inline]
fn add_to_cell<C, I, const PRETTY: bool>(
    cell: &mut Cell<'_>,
    cols: &mut HashMap<C, u16>,
    col: &C,
    value: I,
) -> Result<(), Error>
where
    C: ColHeader,
    I: CellContent,
{
    cell.set_col(col_index(cols, col));
    if PRETTY {
        cell.format_pretty(&value)
    } else {
        cell.format(&value)
    }
}

#[inline]
fn add_row<'a, C, R, I, const PRETTY: bool>(
    cell: &mut Cell<'_>,
    cols: &mut HashMap<C, u16>,
    row: R,
) -> Result<(), Error>
where
    R: Iterator<Item = (&'a C, I)>,
    C: ColHeader + 'a,
    I: CellContent,
{
    for (col, value) in row {
        add_to_cell::<_, _, PRETTY>(cell, cols, col, value)?;
    }

    Ok(())
}

impl<C: ColHeader> DynamicSheet<'static, C> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self::existing(BaseSheet::new(name.into()))
    }
}

impl<'xlsx, C> DynamicSheet<'xlsx, C>
where
    C: ColHeader,
{
    pub(crate) fn existing(mut base: BaseSheet<'xlsx>) -> Self {
        // start at row 1 since we add the header columns in 'finalize'.
        base.row = 1;
        Self {
            base,
            cols_written: false,
            cols: HashMap::new(),
        }
    }

    pub fn predefine_cols<'a, I>(&mut self, cols: I)
    where
        I: IntoIterator<Item = &'a C>,
        C: 'a,
    {
        for col in cols {
            col_index(&mut self.cols, col);
        }
    }

    pub fn name(&self) -> String {
        self.base.name()
    }

    pub fn incr_row(&mut self) {
        self.base.incr_row();
    }

    pub fn add_cell<I>(&mut self, col: &C, value: I) -> Result<(), Error>
    where
        I: CellContent,
    {
        add_to_cell::<_, _, false>(&mut self.base.cell(), &mut self.cols, col, value)
    }

    pub fn add_cell_pretty<I>(&mut self, col: &C, value: I) -> Result<(), Error>
    where
        I: CellContent,
    {
        add_to_cell::<_, _, true>(&mut self.base.cell(), &mut self.cols, col, value)
    }

    pub fn add_row<'a, R, I>(&mut self, row: R) -> Result<(), Error>
    where
        R: IntoIterator<Item = (&'a C, I)>,
        C: 'a,
        I: CellContent,
    {
        add_row::<_, _, _, false>(&mut self.base.cell(), &mut self.cols, row.into_iter())?;
        self.incr_row();
        Ok(())
    }

    pub fn add_rows<'a, R, I>(&mut self, rows: R) -> Result<(), Error>
    where
        R: IntoIterator,
        R::Item: IntoIterator<Item = (&'a C, I)>,
        C: 'a,
        I: CellContent,
    {
        let mut cell = self.base.cell();

        for row in rows {
            add_row::<_, _, _, false>(&mut cell, &mut self.cols, row.into_iter())?;
            cell.incr_row();
        }

        Ok(())
    }

    pub fn finalize(&mut self) -> Result<(), Error> {
        if self.cols_written {
            return Ok(());
        }

        let mut cell = self.base.cell();
        *cell.row = 0;

        for (col, index) in self.cols.iter() {
            cell.set_col(*index);
            col.format_pretty(&mut cell)?;
        }

        self.cols_written = true;

        Ok(())
    }
}
