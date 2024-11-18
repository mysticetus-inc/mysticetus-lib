use std::cell::RefCell;

pub(super) struct InsertRows<R>
where
    R: IntoIterator,
    R::Item: serde::Serialize,
{
    options: InsertRowOptions,
    rows: RowIter<R>,
}

impl<R> serde::Serialize for InsertRows<R>
where
    R: IntoIterator,
    R::Item: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let field_count = self.options.ignore_unknown_values as usize
            + self.options.ignore_unknown_values as usize
            + self.options.trace_id.is_some() as usize
            + 1;

        let mut map = serializer.serialize_map(Some(field_count))?;

        if self.options.ignore_unknown_values {
            map.serialize_entry("ignoreUnknownValues", &true)?;
        }

        if self.options.skip_invalid_rows {
            map.serialize_entry("skipInvalidRows", &true)?;
        }

        if let Some(trace_id) = self.options.trace_id {
            map.serialize_entry("traceId", &trace_id)?;
        }

        map.serialize_entry("rows", &self.rows)?;

        map.end()
    }
}

impl<R> InsertRows<R>
where
    R: IntoIterator,
    R::Item: serde::Serialize,
{
    pub(super) fn new(options: InsertRowOptions, rows: R) -> Self {
        Self {
            options,
            rows: RowIter(RefCell::new(Some(rows))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertRowOptions {
    #[serde(skip_serializing_if = "crate::util::is_false")]
    pub skip_invalid_rows: bool,
    #[serde(skip_serializing_if = "crate::util::is_false")]
    pub ignore_unknown_values: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<uuid::Uuid>,
}

struct RowIter<R>(RefCell<Option<R>>);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RowWrapper<R> {
    insert_id: uuid::Uuid,
    json: R,
}

impl<R> serde::Serialize for RowIter<R>
where
    R: IntoIterator,
    R::Item: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let rows = self.0.take().expect("serialize called twice").into_iter();
        let (low, high) = rows.size_hint();

        let len_hint = match high {
            Some(high) => Some(high),
            _ if low > 0 => Some(low),
            _ => None,
        };
        let mut seq_ser = serializer.serialize_seq(len_hint)?;

        for json in rows {
            seq_ser.serialize_element(&RowWrapper {
                json,
                insert_id: uuid::Uuid::new_v4(),
            })?;
        }

        seq_ser.end()
    }
}
