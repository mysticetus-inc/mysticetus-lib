mod dynamic_sheet;
pub use dynamic_sheet::DynamicSheet;

use crate::cell::Cell;
use crate::col_fmt::ColSums;

/// A base sheet type that contains shared functionality between different sheet implementations
pub(crate) struct BaseSheet<'xlsx> {
    pub(crate) sheet: MaybeOwnedSheet<'xlsx>,
    pub(crate) buf: String,
    pub(crate) row: u32,
    pub(crate) col_sums: Vec<ColSums>,
}

pub(crate) enum MaybeOwnedSheet<'xlsx> {
    Ref(&'xlsx mut rust_xlsxwriter::Worksheet),
    Owned(rust_xlsxwriter::Worksheet),
}

impl MaybeOwnedSheet<'_> {
    fn as_sheet(&self) -> &rust_xlsxwriter::Worksheet {
        match self {
            Self::Ref(refer) => refer,
            Self::Owned(ref owned) => owned,
        }
    }

    fn as_sheet_mut(&mut self) -> &mut rust_xlsxwriter::Worksheet {
        match self {
            Self::Ref(refer) => refer,
            Self::Owned(ref mut owned) => owned,
        }
    }
}

impl BaseSheet<'static> {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        fn inner(name: String) -> BaseSheet<'static> {
            let mut sheet = rust_xlsxwriter::Worksheet::new();

            sheet
                .set_name(crate::clean_sheet_name(name.into()))
                .unwrap();

            BaseSheet {
                row: 0,
                sheet: MaybeOwnedSheet::Owned(sheet),
                buf: String::new(),
                col_sums: Vec::new(),
            }
        }

        inner(name.into())
    }
}

impl<'xlsx> BaseSheet<'xlsx> {
    pub fn name(&self) -> &str {
        self.sheet.as_sheet().name()
    }

    pub(crate) fn existing(sheet: &'xlsx mut rust_xlsxwriter::Worksheet) -> Self {
        Self {
            row: 1,
            sheet: MaybeOwnedSheet::Ref(sheet),
            buf: String::new(),
            col_sums: Vec::new(),
        }
    }

    pub(crate) fn incr_row(&mut self) {
        self.row += 1;
    }

    #[inline]
    pub(crate) fn cell(&mut self) -> Cell<'_> {
        Cell {
            sheet: self.sheet.as_sheet_mut(),
            row: &mut self.row,
            buf: &mut self.buf,
            col_width_sums: &mut self.col_sums,
            col: 0,
            bold: false,
            format: None,
        }
    }
}

impl private::SealedIntoSheet for BaseSheet<'static> {
    fn into_sheet(self) -> rust_xlsxwriter::Worksheet {
        match self.sheet {
            MaybeOwnedSheet::Owned(owned) => owned,
            _ => panic!("'static BaseSheet shouldn't have a mutable reference"),
        }
    }
}

pub trait IntoSheet: private::SealedIntoSheet {}

impl<T> IntoSheet for T where T: private::SealedIntoSheet {}

pub(super) mod private {
    pub trait SealedIntoSheet {
        fn into_sheet(self) -> rust_xlsxwriter::Worksheet;
    }
}
