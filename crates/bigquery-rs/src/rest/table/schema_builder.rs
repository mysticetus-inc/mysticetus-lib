use std::borrow::Cow;

use serde::Serialize;

use super::route;
use crate::rest::Identifier;
use crate::rest::client::InnerClient;

pub struct TableBuilder<'a, D, T> {
    table: Table<'a, D, T>,
    client: &'a InnerClient,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Table<'a, D, T> {
    table_reference: TableReference<'a, D, T>,
    schema: TableSchema<'a>,
    clustering: Option<Clustering<'a>>,
    range_partitioning: Option<RangePartitioning<'a>>,
    time_partitioning: Option<TimePartitioning<'a>>,
}

impl<'a, D, T> TableBuilder<'a, D, T> {
    pub(super) fn init(client: &'a InnerClient, dataset_id: D, table_id: T) -> Self {
        Self {
            client,
            table: Table {
                table_reference: TableReference {
                    project_id: client.project_id(),
                    dataset_id,
                    table_id,
                },
                schema: TableSchema { fields: vec![] },
                clustering: None,
                range_partitioning: None,
                time_partitioning: None,
            },
        }
    }

    pub fn cluster_on_field<F>(mut self, field: F) -> Self
    where
        F: Into<Cow<'a, str>>,
    {
        self.table
            .clustering
            .get_or_insert_default()
            .fields
            .push(field.into());
        self
    }

    pub fn add_field<F: Into<Cow<'a, str>>>(self, field: F) -> TableFieldBuilder<'a, D, T, ()> {
        TableFieldBuilder {
            parent: self,
            field: field.into(),
            mode: FieldMode::Nullable,
            ty: (),
            description: None,
        }
    }
}
impl<D, T> TableBuilder<'_, D, T>
where
    D: Serialize + Identifier,
    T: Serialize,
{
    pub async fn create(self) -> crate::Result<super::Table> {
        let Self { client, table } = self;

        let url = route!(client; "datasets" table.table_reference.dataset_id "tables");

        let table: super::Table = client.post(url, table).await?.json().await?;
        Ok(table)
    }
}

pub struct TableFieldBuilder<'a, D, T, Ty> {
    parent: TableBuilder<'a, D, T>,
    field: Cow<'a, str>,
    mode: FieldMode,
    ty: Ty,
    description: Option<Cow<'a, str>>,
}

impl<'a, D, T, Ty> TableFieldBuilder<'a, D, T, Ty> {
    pub fn required(mut self) -> Self {
        self.mode = FieldMode::Required;
        self
    }
    pub fn repeated(mut self) -> Self {
        self.mode = FieldMode::Repeated;
        self
    }
    pub fn description<S>(mut self, desc: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.description = Some(desc.into());
        self
    }

    pub fn cluster_on(mut self) -> Self {
        self.parent
            .table
            .clustering
            .get_or_insert_default()
            .fields
            .push(self.field.clone());
        self
    }
}

macro_rules! impl_field_type_fns {
    ($($fn_name:ident($variant:ident)),* $(,)?) => {
        $(
            #[inline]
            pub fn $fn_name(self) -> TableFieldBuilder<'a, D, T, FieldType> {
                self.field_type(FieldType::$variant)
            }
        )*
    }
}

impl<'a, D, T> TableFieldBuilder<'a, D, T, ()> {
    #[inline]
    fn field_type(self, ty: FieldType) -> TableFieldBuilder<'a, D, T, FieldType> {
        TableFieldBuilder { ty, ..self }
    }

    impl_field_type_fns! {
        string(String),
        bytes(Bytes),
        integer(Integer),
        float(Float),
        boolean(Bool),
        timestamp(Timestamp),
        date(Date),
        time(Time),
        datetime(DateTime),
        geography(Geography),
        numeric(Numeric),
        big_numeric(BigNumeric),
        // record(Record),
    }
}

macro_rules! impl_time_partition_fns {
    ($($fn_name:ident($variant:ident)),* $(,)?) => {
        $(
            #[inline]
            pub fn $fn_name(self) -> Self {
                self.partition_by_time(TimePartitionType::$variant)
            }
        )*
    }
}

impl<'a, D, T> TableFieldBuilder<'a, D, T, FieldType> {
    pub fn partition_by_range(mut self) -> Self {
        assert_eq!(
            self.ty,
            FieldType::Integer,
            "cannot partition by range on non-integer fields"
        );
        assert!(
            self.mode != FieldMode::Repeated,
            "cannot partition on repeated fields"
        );
        self.parent.table.range_partitioning = Some(RangePartitioning {
            field: self.field.clone(),
        });

        self
    }

    fn partition_by_time(mut self, ty: TimePartitionType) -> Self {
        assert!(
            matches!(self.ty, FieldType::Timestamp | FieldType::Date),
            "cannot partition by time on a non timestamp/date field"
        );
        assert!(
            self.mode != FieldMode::Repeated,
            "cannot partition on repeated fields"
        );
        self.parent.table.time_partitioning = Some(TimePartitioning {
            ty,
            field: Some(self.field.clone()),
        });
        self
    }

    impl_time_partition_fns! {
        partition_by_hour(Hour),
        partition_by_day(Day),
        partition_by_month(Month),
        partition_by_year(Day),
    }

    pub fn finish_field(self) -> TableBuilder<'a, D, T> {
        let Self {
            mut parent,
            field,
            mode,
            ty,
            description,
        } = self;

        parent.table.schema.fields.push(TableField {
            name: field,
            ty,
            mode,
            description,
        });

        parent
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableReference<'a, D, T> {
    project_id: &'a str,
    dataset_id: D,
    table_id: T,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RangePartitioning<'a> {
    field: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimePartitioning<'a> {
    #[serde(rename = "type")]
    ty: TimePartitionType,
    field: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimePartitionType {
    Day,
    Hour,
    Month,
    Year,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct TableSchema<'a> {
    fields: Vec<TableField<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct Clustering<'a> {
    fields: Vec<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableField<'a> {
    name: Cow<'a, str>,
    #[serde(rename = "type")]
    ty: FieldType,
    mode: FieldMode,
    description: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FieldMode {
    Nullable,
    Repeated,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FieldType {
    String,
    Bytes,
    Integer,
    Float,
    Bool,
    Timestamp,
    Date,
    Time,
    DateTime,
    Geography,
    Numeric,
    BigNumeric,
    Record,
}
