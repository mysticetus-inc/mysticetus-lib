use std::fmt;
use std::path::Path;

use crate::error::Error;
use crate::sheets;

/// Thin wrapper around an [`xlsxwriter::Workbook`].
#[must_use = "must be written to a file/buffer"]
pub struct ExcelFile {
    book: rust_xlsxwriter::Workbook,
}

impl fmt::Debug for ExcelFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugSheets<'a>(&'a Vec<rust_xlsxwriter::Worksheet>);

        impl fmt::Debug for DebugSheets<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list()
                    .entries(self.0.iter().map(|sheet| sheet.name()))
                    .finish()
            }
        }

        formatter
            .debug_struct("ExcelFile")
            .field("sheets", &DebugSheets(self.book.worksheets()))
            .finish_non_exhaustive()
    }
}

impl ExcelFile {
    pub fn new() -> Self {
        Self {
            book: rust_xlsxwriter::Workbook::new(),
        }
    }

    #[inline]
    pub fn write_to_buffer(mut self) -> Result<Vec<u8>, Error> {
        self.book.save_to_buffer().map_err(Error::from)
    }

    #[inline]
    pub fn write_to<P: AsRef<Path>>(self, path: P) -> Result<(), Error> {
        fn inner(
            mut book: rust_xlsxwriter::Workbook,
            path: &Path,
        ) -> Result<(), rust_xlsxwriter::XlsxError> {
            book.save(path)
        }

        inner(self.book, path.as_ref()).map_err(Error::from)
    }

    pub fn sort_sheets_by<F>(&mut self, mut sort_fn: F)
    where
        F: FnMut(&str, &str) -> std::cmp::Ordering,
    {
        self.book
            .worksheets_mut()
            .sort_by(|a, b| sort_fn(a.name(), b.name()));
    }
    pub fn sort_sheets_by_name(&mut self) {
        self.sort_sheets_by(|a, b| a.cmp(b));
    }

    pub fn append_sheet<S>(&mut self, sheet: S)
    where
        S: sheets::IntoSheet,
    {
        self.book.push_worksheet(sheet.into_sheet());
    }

    fn get_or_insert_sheet(&mut self, sheet_name: &str) -> &mut rust_xlsxwriter::Worksheet {
        let sheets = self.book.worksheets_mut();

        // edge-case of the current borrow-checker, need to use indexes to avoid the borrow-checker
        // from thinking the mutable borrow lasts the entire function, therefore making the
        // needed push illegal
        if let Some(index) = sheets
            .iter_mut()
            .position(|sheet| sheet.name() == sheet_name)
        {
            &mut sheets[index]
        } else {
            let mut new = rust_xlsxwriter::Worksheet::new();
            // 'clean_sheet_name' ensures the name is a valid excel sheet name
            new.set_name(crate::clean_sheet_name(sheet_name.into()))
                .unwrap();
            sheets.push(new);
            sheets.last_mut().unwrap()
        }
    }

    fn get_base_sheet(&mut self, sheet_name: &str) -> sheets::BaseSheet<'_> {
        sheets::BaseSheet::existing(self.get_or_insert_sheet(sheet_name))
    }

    pub fn get_dynamic_sheet<C>(&mut self, sheet: &str) -> sheets::DynamicSheet<'_, C>
    where
        C: crate::content::ColHeader + ?Sized,
    {
        sheets::DynamicSheet::existing(self.get_base_sheet(sheet))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandleTemp {
    /// Delete the temproary file.
    Delete,
    /// Leave the temp file at the path specified.
    Leave,
}
