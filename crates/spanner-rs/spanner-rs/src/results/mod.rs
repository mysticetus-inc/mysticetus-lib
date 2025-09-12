use generic_array::{ArrayLength, GenericArray};
use protos::spanner::ResultSetMetadata;

use crate::Field;
use crate::column::InvalidColumnIndex;
use crate::error::MissingTypeInfo;
use crate::queryable::Queryable;

pub mod iter;
pub mod stats;
pub mod streaming;

pub use iter::ResultIter;
pub use streaming::StreamingRead;

#[derive(Clone)]
pub struct FieldIndex<Cols: ArrayLength> {
    fields: GenericArray<(usize, Field), Cols>,
}

impl<Cols: ArrayLength> std::fmt::Debug for FieldIndex<Cols> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.fields.iter()).finish()
    }
}

struct Indexes {
    data: usize,
    expected: usize,
}

impl<Cols: ArrayLength> FieldIndex<Cols> {
    #[inline]
    pub const fn len(&self) -> usize {
        <Cols as typenum::Unsigned>::USIZE
    }

    pub(crate) fn from_struct_type<Q: Queryable<NumColumns = Cols>>(
        raw: protos::spanner::StructType,
    ) -> Result<Self, MissingTypeInfo> {
        let mut fields = raw
            .fields
            .into_iter()
            .enumerate()
            .map(|(data, proto)| {
                Field::from_proto(proto).map(|field| (Indexes { data, expected: 0 }, field))
            })
            .collect::<Result<GenericArray<(Indexes, Field), Q::NumColumns>, MissingTypeInfo>>()?;

        Ok(Self { fields })
    }

    pub(crate) fn from_result_set_meta<Q: Queryable<NumColumns = Cols>>(
        raw: Option<ResultSetMetadata>,
    ) -> Result<Self, MissingTypeInfo> {
        raw.and_then(|meta| meta.row_type)
            .ok_or_else(MissingTypeInfo::missing)
            .and_then(Self::from_struct_type::<Q>)
    }

    pub fn get_field_at_index(&self, index: usize) -> Result<&Field, InvalidColumnIndex> {
        self.fields
            .get(index)
            .ok_or_else(|| InvalidColumnIndex::new_explicit(index, self.fields.len()))
    }
}

/// A raw row from spanner, with a reference to the type information for each column.
#[derive(Debug)]
pub struct RawRow<'a, Cols: ArrayLength> {
    index: &'a FieldIndex<Cols>,
    row: Vec<protos::protobuf::Value>,
}

impl<'a, Cols: ArrayLength> RawRow<'a, Cols> {
    pub(crate) fn new(index: &'a FieldIndex<Cols>, row: Vec<protos::protobuf::Value>) -> Self {
        Self { index, row }
    }
}

impl<Cols: ArrayLength> RawRow<'_, Cols> {
    pub fn decode_at_index<F, E, T>(&mut self, index: usize, decoder: F) -> crate::Result<T>
    where
        F: FnOnce(&crate::Field, crate::Value) -> Result<T, E>,
        E: Into<crate::Error>,
    {
        let value = crate::Value::from_kind_opt(self.row[index].kind.take());
        let field = self.index.get_field_at_index(index)?;

        (decoder)(field, value).map_err(Into::into)
    }
}
