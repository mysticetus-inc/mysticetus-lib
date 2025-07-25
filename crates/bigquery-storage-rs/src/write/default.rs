//! Special handling needed for getting the '_default' stream table schema
//!
//! For some reason, `GetWriteStream` won't work on the default stream, and attemping
//! to append rows without a proto schema fails (or with 0 rows), so we need to use the
//! REST API to get the schema first.
//!
//! This follows the same method of falling back to the REST API used internally by the Google
//! written Java BigQuery Storage API. Seems like bad design to me.

use http::HeaderValue;
use protos::bigquery_storage::table_field_schema::{Mode, Type};
use protos::bigquery_storage::{TableFieldSchema, TableSchema};
use serde::Deserialize;

/// Using the REST API, loads the table schema that would normally come from creating a
/// [`WriteStream`].
///
/// Takes the qualified path to a table, in the form:
/// 'projects/{project_id}/datasets/{dataset_id}/tables/{table_id}'
///
/// Also takes a reference to an [`Auth`] provider, in order to get a token.
///
/// Lastly, the [`reqwest::Client`] can provided if one already exists, otherwise a new one
/// will be built soley for this request.
///
/// [`WriteStream`]: [`protos::bigquery_storage::WriteStream`]
/// [`Auth`]: [`gcp_auth_channel::Auth`]
pub async fn load_default_schema(
    qualified_path: &str,
    auth: &gcp_auth_provider::Auth,
    client: Option<&reqwest::Client>,
) -> Result<TableSchema, crate::Error> {
    async fn get_schema(
        url: String,
        token: HeaderValue,
        client: Option<&reqwest::Client>,
    ) -> reqwest::Result<TableSchema> {
        let req_builder = match client {
            Some(client) => client.get(url),
            _ => reqwest::Client::new().get(url),
        };

        let RestTable { schema } = req_builder
            .header(reqwest::header::AUTHORIZATION, token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(schema)
    }

    let url = format!(
        "https://bigquery.googleapis.com/bigquery/v2/{}",
        qualified_path
    );

    let token = auth.get_header().into_header().await?.header;

    get_schema(url, token, client)
        .await
        .map_err(|err| crate::Error::Status(tonic::Status::internal(err.to_string())))
}

#[derive(Deserialize)]
struct RestTable {
    #[serde(deserialize_with = "deserialize_table_schema")]
    schema: protos::bigquery_storage::TableSchema,
}

fn deserialize_table_schema<'de, D>(deserializer: D) -> Result<TableSchema, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let RestTableSchema { fields } = Deserialize::deserialize(deserializer)?;
    Ok(TableSchema { fields })
}

#[derive(serde::Deserialize)]
struct RestTableSchema {
    #[serde(deserialize_with = "deserialize_field_array")]
    fields: Vec<TableFieldSchema>,
}

fn deserialize_field_array<'de, D>(deserializer: D) -> Result<Vec<TableFieldSchema>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_seq(Visitor)
}

struct Visitor;

impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Vec<TableFieldSchema>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a table field schema")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut buf = seq.size_hint().map(Vec::with_capacity).unwrap_or_default();

        while let Some(field) = seq.next_element::<RestTableFieldSchema>()? {
            let converted = TableFieldSchema {
                name: field.name,
                mode: field.mode as i32,
                r#type: field.r#type as i32,
                ..Default::default()
            };

            buf.push(converted);
        }

        Ok(buf)
    }
}

#[derive(serde::Deserialize)]
struct RestTableFieldSchema {
    name: String,
    r#type: RestType,
    #[serde(default)]
    mode: RestMode,
}

// use the original [`Type`] and [`Mode`] definitions to determine the integer repr,
// that way they stay in sync (and wont need an explicit conversion function)

#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[repr(i32)]
enum RestType {
    String = Type::String as i32,
    Bytes = Type::Bytes as i32,
    #[serde(alias = "INT64")]
    Integer = Type::Int64 as i32,
    #[serde(alias = "FLOAT64")]
    Float = Type::Double as i32,
    #[serde(alias = "BOOL")]
    Boolean = Type::Bool as i32,
    Timestamp = Type::Timestamp as i32,
    Date = Type::Date as i32,
    Time = Type::Time as i32,
    Datetime = Type::Datetime as i32,
    Geography = Type::Geography as i32,
    Numeric = Type::Numeric as i32,
    BigNumeric = Type::Bignumeric as i32,
    #[serde(alias = "STRUCT")]
    Record = Type::Struct as i32,
}

#[derive(Clone, Default, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[repr(i32)]
enum RestMode {
    #[default]
    Nullable = Mode::Nullable as i32,
    Required = Mode::Required as i32,
    Repeated = Mode::Repeated as i32,
}
