use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::error::Error;
use crate::sheets;

/// Thin wrapper around an [`xlsxwriter::Workbook`].
#[must_use = "must be written to a file/buffer"]
pub struct ExcelFile {
    book: Box<rust_xlsxwriter::Workbook>,
    index: HashMap<Box<str>, usize>,
}

impl fmt::Debug for ExcelFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExcelFile")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl ExcelFile {
    pub fn new() -> Self {
        Self {
            book: Box::new(rust_xlsxwriter::Workbook::new()),
            index: HashMap::new(),
        }
    }

    #[inline]
    pub fn write_to_buffer(mut self) -> Result<Vec<u8>, Error> {
        self.book.save_to_buffer().map_err(Error::from)
    }

    #[inline]
    pub fn write_to<W: std::io::Write + std::io::Seek + Send>(
        mut self,
        writer: W,
    ) -> Result<(), Error> {
        self.book.save_to_writer(writer)?;
        Ok(())
    }

    #[inline]
    pub fn write_to_file<P: AsRef<Path>>(self, path: P) -> Result<(), Error> {
        fn inner(
            mut book: Box<rust_xlsxwriter::Workbook>,
            path: &Path,
        ) -> Result<(), rust_xlsxwriter::XlsxError> {
            book.save(path)
        }

        inner(self.book, path.as_ref()).map_err(Error::from)
    }

    pub fn append_sheet<S>(&mut self, sheet: S)
    where
        S: sheets::IntoSheet,
    {
        let sheet = sheet.into_sheet();
        self.index.insert(
            sheet.name().into_boxed_str(),
            self.book.worksheets_mut().len(),
        );
        self.book.worksheets_mut().push(*sheet);
    }

    fn get_or_insert_sheet(&mut self, sheet_name: &str) -> &mut rust_xlsxwriter::Worksheet {
        let sheets = self.book.worksheets_mut();

        // edge-case of the current borrow-checker, need to use indexes to avoid the borrow-checker
        // from thinking the mutable borrow lasts the entire function, therefore making the
        // needed push illegal
        if let Some(index) = self.index.get(sheet_name).copied() {
            &mut sheets[index]
        } else {
            let mut new = rust_xlsxwriter::Worksheet::new();
            let name = crate::clean_sheet_name(sheet_name.into());
            // 'clean_sheet_name' ensures the name is a valid excel sheet name
            new.set_name(&*name).unwrap();
            self.index
                .insert(name.into_owned().into_boxed_str(), sheets.len());
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

    pub fn get_simple_sheet(&mut self, sheet: &str) -> sheets::SimpleSheet<'_> {
        sheets::SimpleSheet::new(self.get_base_sheet(sheet))
    }
}

impl<S> Extend<S> for ExcelFile
where
    S: crate::sheets::IntoSheet,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = S>,
    {
        for sheet in iter {
            self.append_sheet(sheet);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandleTemp {
    /// Delete the temproary file.
    Delete,
    /// Leave the temp file at the path specified.
    Leave,
}
