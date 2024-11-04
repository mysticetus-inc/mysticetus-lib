use std::collections::BTreeMap;
use std::convert::Infallible;
use std::num::NonZeroUsize;
use std::sync::Arc;

// use genco::prelude::Rust;
use spanner_rs::convert::SpannerEncode;
use spanner_rs::error::{ConvertError, FromError};
use spanner_rs::info::Database;
use spanner_rs::ty::SpannerType;
use spanner_rs::{Client, FromSpanner, IntoSpanner, ResultIter, Scalar, Session, Value};

const QUERY: &str = "SELECT
  COLS.TABLE_NAME,
  COLS.COLUMN_NAME, 
  COLS.ORDINAL_POSITION, 
  COLS.COLUMN_DEFAULT, 
  COLS.IS_NULLABLE,
  COLS.SPANNER_TYPE, 
  COLS.IS_GENERATED, 
  COLS.SPANNER_STATE,
  IDX.INDEX_NAME,
  IDX.INDEX_TYPE,
  IDX.COLUMN_ORDERING 
FROM 
  INFORMATION_SCHEMA.COLUMNS AS COLS 
FULL OUTER JOIN 
  INFORMATION_SCHEMA.INDEX_COLUMNS AS IDX 
ON 
  COLS.COLUMN_NAME = IDX.COLUMN_NAME AND COLS.TABLE_NAME = IDX.TABLE_NAME
WHERE 
  COLS.TABLE_SCHEMA = \"\"
  AND COLS.TABLE_NAME IN (
    SELECT
      TABLE_NAME 
    FROM 
      INFORMATION_SCHEMA.TABLES 
    WHERE 
      TABLE_TYPE = \"BASE TABLE\"
  )
ORDER BY 
  COLS.TABLE_NAME,
  COLS.ORDINAL_POSITION";

#[derive(Debug)]
pub struct InformationSchema {
    tables: BTreeMap<Arc<str>, Table>,
}

impl InformationSchema {
    pub async fn read(client: &mut Client) -> anyhow::Result<Self> {
        let mut sess = client.create_session(None).await?;
        let res = Self::read_from_session(&mut sess).await;
        sess.delete().await?;
        res
    }

    pub async fn read_from_session(sess: &mut Session) -> anyhow::Result<Self> {
        let iter = sess
            .execute_sql::<RawColumnView>(QUERY.to_owned(), None)
            .await?;

        let mut dst = Self {
            tables: BTreeMap::new(),
        };

        for res in iter {
            let col = res?;
            dst.push_col(col)?;
        }

        Ok(dst)
    }

    fn push_col(&mut self, raw: RawColumnView) -> anyhow::Result<()> {
        let RawColumnView {
            table_name,
            column_name,
            ordinal_position,
            column_default,
            is_nullable,
            spanner_type,
            is_generated,
            spanner_state,
            index_type,
            column_ordering,
            ..
        } = raw;

        let name = Arc::from(table_name);
        let table = self.tables.entry(name).or_insert_with_key(|k| Table {
            name: Arc::clone(k),
            columns: Vec::new(),
        });

        let index = match (index_type, column_ordering) {
            (Some(index_type), Some(ordering)) => Some(Index {
                index_type,
                ordering,
            }),
            (Some(index_type), None) => Some(Index {
                index_type,
                ordering: Ordering::Asc,
            }),
            (None, _) => None,
        };

        table.columns.push(Column {
            name: column_name,
            ordinal_position,
            column_default,
            spanner_state,
            is_nullable: matches!(is_nullable, IsNullable::Yes),
            spanner_type,
            is_generated: matches!(is_generated, IsGenerated::Always),
            index,
        });

        Ok(())
    }

    pub async fn read_from_db<S: AsRef<str>>(db: Database<S>) -> anyhow::Result<Self>
    where
        data_structures::shared::Shared<str>: From<S>,
    {
        let mut client = db.build_client().await?;
        Self::read(&mut client).await
    }
}

#[derive(Debug)]
pub struct Table {
    name: Arc<str>,
    columns: Vec<Column>,
}

impl genco::tokens::FormatInto<Rust> for &Table {
    fn format_into(self, tokens: &mut genco::Tokens<Rust>) {
        genco::quote_in! { *tokens =>
            #[derive(Debug, Clone, PartialEq, spanner_rs::Table)]
            pub struct #(self.name) {
                #(self.col)
            }
        }
    }
}

#[derive(Debug)]
pub struct Column {
    name: String,
    ordinal_position: NonZeroUsize,
    column_default: Option<String>,
    spanner_state: SpannerState,
    is_nullable: bool,
    spanner_type: DataType,
    is_generated: bool,
    index: Option<Index>,
}

impl genco::tokens::FormatInto<Rust> for &Column {
    fn format_into(self, tokens: &mut genco::Tokens<Rust>) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Index {
    index_type: IndexType,
    ordering: Ordering,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IndexType {
    PrimaryKey,
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Ordering {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Max,
    Sized(usize),
}

impl Size {
    fn fmt_into(&self, dst: &mut String) {
        match self {
            Self::Max => dst.push_str("MAX"),
            Self::Sized(size) => dst.push_str(itoa::Buffer::new().format(*size)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataType {
    Int64,
    String(Size),
    Bytes(Size),
    Json(Size),
    Float64,
    Bool,
    Date,
    Timestamp,
    Array(Box<DataType>),
}

impl SpannerType for DataType {
    const TYPE: &'static spanner_rs::Type = &spanner_rs::Type::Scalar(Scalar::String);
}

impl IntoSpanner for DataType {
    fn into_value(self) -> Value {
        let mut dst = String::with_capacity(32);
        self.fmt_into(&mut dst);
        dst.into_value()
    }
}

impl FromSpanner for DataType {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let s = value.into_string::<Self>()?;
        Self::parse_from(&s).map_err(ConvertError::from)
    }
}

fn parse_size(s: &str) -> Result<Size, FromError> {
    if s.eq_ignore_ascii_case("MAX") {
        Ok(Size::Max)
    } else {
        s.parse()
            .map(Size::Sized)
            .map_err(|err| FromError::from_error::<DataType>(err))
    }
}

fn parse_array(s: &str) -> Result<DataType, FromError> {
    const LEADING: &str = "ARRAY<";

    if s.len() <= LEADING.len() || !s[..LEADING.len()].eq_ignore_ascii_case(LEADING) {
        return Err(FromError::from_anyhow::<DataType>(anyhow::anyhow!(
            "invalid string"
        )));
    }

    let rem = &s[LEADING.len()..];

    let end = rem
        .find('>')
        .ok_or_else(|| FromError::from_anyhow::<DataType>(anyhow::anyhow!("invalid string")))?;

    let elem = DataType::parse_from(&rem[..end])?;
    Ok(DataType::Array(Box::new(elem)))
}

fn parse_size_type(s: &str) -> Option<Result<DataType, FromError>> {
    if s.contains("<") {
        return None;
    }

    let start = s.find('(')?;
    let end = s.find(')')?;

    let leading = &s[..start];

    let size = match parse_size(&s[start + 1..end]) {
        Ok(size) => size,
        Err(err) => return Some(Err(err)),
    };

    if leading.eq_ignore_ascii_case("BYTES") {
        Some(Ok(DataType::Bytes(size)))
    } else if leading.eq_ignore_ascii_case("JSON") {
        Some(Ok(DataType::Json(size)))
    } else if leading.eq_ignore_ascii_case("STRING") {
        Some(Ok(DataType::String(size)))
    } else {
        Some(Err(FromError::from_anyhow::<DataType>(anyhow::anyhow!(
            "unknown data type '{leading}'"
        ))))
    }
}

impl DataType {
    fn parse_from(s: &str) -> Result<Self, FromError> {
        const SIMPLE: &[(&str, DataType)] = &[
            ("INT64", DataType::Int64),
            ("FLOAT64", DataType::Float64),
            ("BOOL", DataType::Bool),
            ("DATE", DataType::Date),
            ("TIMESTAMP", DataType::Timestamp),
        ];

        if let Some(dt) = SIMPLE.iter().find_map(|(simple, val)| {
            if simple.eq_ignore_ascii_case(s) {
                Some(val.clone())
            } else {
                None
            }
        }) {
            return Ok(dt);
        }

        if let Some(res) = parse_size_type(s) {
            return res;
        }

        parse_array(s)
    }

    fn fmt_into(&self, dst: &mut String) {
        match self {
            Self::Int64 => dst.push_str("INT64"),
            Self::Float64 => dst.push_str("FLOAT64"),
            Self::Bool => dst.push_str("BOOL"),
            Self::Date => dst.push_str("DATE"),
            Self::Timestamp => dst.push_str("TIMESTAMP"),
            Self::String(size) => {
                dst.push_str("STRING(");
                size.fmt_into(dst);
                dst.push_str(")");
            }
            Self::Bytes(size) => {
                dst.push_str("BYTES(");
                size.fmt_into(dst);
                dst.push_str(")");
            }
            Self::Json(size) => {
                dst.push_str("JSON(");
                size.fmt_into(dst);
                dst.push_str(")");
            }
            Self::Array(elem) => {
                dst.push_str("ARRAY<");
                elem.fmt_into(dst);
                dst.push_str(">");
            }
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpannerState {
    Committed,
    WriteOnly,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IsNullable {
    Yes,
    No,
}

#[test]
fn test_wtf() {
    use serde::de::value::Error;
    use serde::de::{Deserialize, IntoDeserializer};
    let res: Result<_, Error> = IsNullable::deserialize("NO".into_deserializer());

    println!("{res:#?}");
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IsGenerated {
    Never,
    Always,
}

#[derive(Debug, spanner_rs::Table)]
pub struct RawColumnView {
    #[spanner(pk = 0)]
    table_name: String,
    #[spanner(pk = 1)]
    column_name: String,
    ordinal_position: std::num::NonZeroUsize,
    column_default: core::option::Option<String>,
    #[spanner(with_serde_as = "String")]
    is_nullable: IsNullable,
    spanner_type: DataType,
    #[spanner(with_serde_as = "String")]
    is_generated: IsGenerated,
    #[spanner(with_serde_as = "String")]
    spanner_state: SpannerState,
    #[spanner(with_serde_as = "String")]
    index_name: Option<IndexType>,
    #[spanner(with_serde_as = "String")]
    index_type: Option<IndexType>,
    #[spanner(with_serde_as = "String")]
    column_ordering: Option<Ordering>,
}
