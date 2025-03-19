use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::ops::Bound;

use protos::protobuf::ListValue;
use protos::spanner::key_range::{EndKeyType, StartKeyType};
use protos::spanner::{self, KeyRange};

use crate::error::ConvertError;
use crate::pk::IntoPartialPkParts;
use crate::private::SealedToKey;
use crate::results::{ResultIter, StreamingRead};
use crate::{PrimaryKey, Table};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize)]
pub struct WriteBuilder<T: Table> {
    rows: Vec<ListValue>,
    cells: usize,
    _marker: PhantomData<T>,
}

impl<T: Table> WriteBuilder<T> {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn add_row(&mut self, row: T) -> Result<(), ConvertError> {
        let values = row.into_row()?.0;
        self.cells += values.len();
        self.rows.push(ListValue { values });

        Ok(())
    }

    pub fn add_rows<I>(&mut self, rows: I) -> Result<(), ConvertError>
    where
        I: IntoIterator<Item = T>,
    {
        let iter = rows.into_iter();
        let (low, high) = iter.size_hint();

        self.rows.reserve(high.unwrap_or(low));

        for row in iter {
            let values = row.into_row()?.0;
            self.cells += values.len();
            self.rows.push(ListValue { values });
        }

        Ok(())
    }

    pub fn with_row_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            cells: 0,
            _marker: PhantomData,
        }
    }

    pub(crate) fn into_proto(self) -> spanner::mutation::Write {
        spanner::mutation::Write {
            table: T::NAME.to_owned(),
            columns: crate::util::table_col_names::<T>(),
            values: self.rows,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeySet<T> {
    keys: Vec<ListValue>,
    ranges: Vec<KeyRange>,
    limit: Option<NonZeroU32>,
    _marker: PhantomData<T>,
}

/// Trait for types that can be turned into a [`KeySet<T>`].
///
/// Currently only used to streamline using both [`KeySet<T>`] and [`&mut KeySet<T>`].
///
/// [`&mut KeySet<T>`]: KeySet<T>
pub trait IntoKeySet<T: Table>: Sized {
    /// Turn 'self' into a [`KeySet<T>`].
    fn into_key_set(self) -> KeySet<T>;
}

impl<T: Table> IntoKeySet<T> for KeySet<T> {
    #[inline]
    fn into_key_set(self) -> KeySet<T> {
        self
    }
}

impl<T: Table> IntoKeySet<T> for &mut KeySet<T> {
    #[inline]
    fn into_key_set(self) -> KeySet<T> {
        self.take()
    }
}

impl<T> Default for KeySet<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Table> FromIterator<T::Pk> for KeySet<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T::Pk>,
    {
        let keys = iter
            .into_iter()
            .map(|pk| pk.into_parts().to_key())
            .collect();

        Self {
            keys,
            ranges: Vec::new(),
            limit: None,
            _marker: PhantomData,
        }
    }
}

impl<T: Table> Extend<T::Pk> for KeySet<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T::Pk>,
    {
        let mapped_iter = iter.into_iter().map(|pk| pk.into_parts().to_key());

        self.keys.extend(mapped_iter);
    }
}

impl<T> KeySet<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            keys: Vec::new(),
            ranges: Vec::new(),
            limit: None,
            _marker: PhantomData,
        }
    }

    pub const fn take(&mut self) -> Self {
        std::mem::replace(self, KeySet::new())
    }

    #[inline]
    pub fn with_capacity(keys: usize, ranges: usize) -> Self {
        Self {
            keys: Vec::with_capacity(keys),
            ranges: Vec::with_capacity(ranges),
            limit: None,
            _marker: PhantomData,
        }
    }

    /// Sets a limit on how many key-range derived rows will be retrieved.
    ///
    /// The real row limit that spanner will receive is this number, plus
    /// the number of explcit, full primary keys added to this [`KeySet`]
    #[inline]
    pub fn limit(&mut self, rows: u32) -> &mut Self {
        self.limit = NonZeroU32::new(rows);
        self
    }

    #[inline]
    pub fn get_limit(&self) -> Option<u32> {
        self.limit
            .map(|n| n.get().saturating_add(self.keys.len() as u32))
    }

    pub(crate) fn into_proto(self) -> spanner::KeySet {
        spanner::KeySet {
            keys: self.keys,
            ranges: self.ranges,
            all: false,
        }
    }

    pub(crate) fn to_proto(&mut self) -> spanner::KeySet {
        self.take().into_proto()
    }
}

fn convert_to_range<R, S, T: Table>(range: R) -> KeyRange
where
    R: OwnedRangeBounds<S>,
    S: IntoPartialPkParts<T>,
{
    let (start, end) = range.into_bounds();

    let start = match start {
        Bound::Included(incl) => StartKeyType::StartClosed(incl.into_partial_parts().to_key()),
        Bound::Excluded(excl) => StartKeyType::StartOpen(excl.into_partial_parts().to_key()),
        Bound::Unbounded => StartKeyType::StartClosed(ListValue { values: vec![] }),
    };

    let end = match end {
        Bound::Included(incl) => EndKeyType::EndClosed(incl.into_partial_parts().to_key()),
        Bound::Excluded(excl) => EndKeyType::EndOpen(excl.into_partial_parts().to_key()),
        Bound::Unbounded => EndKeyType::EndClosed(ListValue { values: vec![] }),
    };

    KeyRange {
        start_key_type: Some(start),
        end_key_type: Some(end),
    }
}

impl<T: Table> KeySet<T> {
    #[inline]
    pub fn add_key(&mut self, key: T::Pk) -> &mut Self {
        self.keys.push(key.into_parts().to_key());
        self
    }

    pub async fn read<C>(&mut self, conn: &C) -> crate::Result<ResultIter<T>>
    where
        C: crate::ReadableConnection,
    {
        conn.connection_parts().read_key_set(self.take()).await
    }

    pub async fn streaming_read<C>(&mut self, conn: &C) -> crate::Result<StreamingRead<T>>
    where
        C: crate::ReadableConnection,
    {
        conn.connection_parts()
            .streaming_read_key_set(self.take())
            .await
    }

    pub fn add_keys<I>(&mut self, keys: I) -> &mut Self
    where
        I: IntoIterator<Item = T::Pk>,
    {
        let iter = keys.into_iter().map(|key| key.into_parts().to_key());
        self.keys.extend(iter);
        self
    }

    pub fn add_range<R, S>(&mut self, range: R) -> &mut Self
    where
        R: OwnedRangeBounds<S>,
        S: IntoPartialPkParts<T>,
    {
        self.ranges.push(convert_to_range(range));
        self
    }

    pub fn start_at<S>(&mut self, start_at: S) -> KeyRangeBuilder<'_, T>
    where
        S: IntoPartialPkParts<T>,
    {
        self.start_inner(StartKeyType::StartClosed(
            start_at.into_partial_parts().to_key(),
        ))
    }

    pub fn start_after<S>(&mut self, start_at: S) -> KeyRangeBuilder<'_, T>
    where
        S: IntoPartialPkParts<T>,
    {
        self.start_inner(StartKeyType::StartOpen(
            start_at.into_partial_parts().to_key(),
        ))
    }

    pub fn from_start(&mut self) -> KeyRangeBuilder<'_, T> {
        self.start_inner(StartKeyType::StartClosed(ListValue::default()))
    }

    fn start_inner(&mut self, start: StartKeyType) -> KeyRangeBuilder<'_, T> {
        KeyRangeBuilder {
            key_set: self,
            start,
        }
    }
}

pub struct KeyRangeBuilder<'a, T> {
    key_set: &'a mut KeySet<T>,
    start: StartKeyType,
}

impl<'a, T: Table> KeyRangeBuilder<'a, T> {
    fn end_inner(self, end: EndKeyType) -> &'a mut KeySet<T> {
        self.key_set.ranges.push(KeyRange {
            start_key_type: Some(self.start),
            end_key_type: Some(end),
        });
        self.key_set
    }

    pub fn end_before<S>(self, end_before: S) -> &'a mut KeySet<T>
    where
        S: IntoPartialPkParts<T>,
    {
        self.end_inner(EndKeyType::EndOpen(
            end_before.into_partial_parts().to_key(),
        ))
    }

    pub fn end_at<S>(self, end_before: S) -> &'a mut KeySet<T>
    where
        S: IntoPartialPkParts<T>,
    {
        self.end_inner(EndKeyType::EndClosed(
            end_before.into_partial_parts().to_key(),
        ))
    }

    pub fn to_end(self) -> &'a mut KeySet<T> {
        self.end_inner(EndKeyType::EndClosed(ListValue::default()))
    }
}

/// Essentially the same as [`std::ops::RangeBounds`], but consumes 'self' to return owned bounds.
///
/// Implemented for every `RangeXXXXXX` type in [`std::ops`].
pub trait OwnedRangeBounds<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>);
}

impl<T> OwnedRangeBounds<T> for std::ops::RangeFull {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        (Bound::Unbounded, Bound::Unbounded)
    }
}

impl<T> OwnedRangeBounds<T> for std::ops::Range<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        (Bound::Included(self.start), Bound::Excluded(self.end))
    }
}

impl<T> OwnedRangeBounds<T> for std::ops::RangeTo<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        (Bound::Unbounded, Bound::Excluded(self.end))
    }
}

impl<T> OwnedRangeBounds<T> for std::ops::RangeInclusive<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        let (start, end) = self.into_inner();
        (Bound::Included(start), Bound::Included(end))
    }
}

impl<T> OwnedRangeBounds<T> for std::ops::RangeToInclusive<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        (Bound::Unbounded, Bound::Included(self.end))
    }
}

impl<T> OwnedRangeBounds<T> for std::ops::RangeFrom<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        (Bound::Included(self.start), Bound::Unbounded)
    }
}
