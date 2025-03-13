use std::borrow::Cow;

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Discovery {
    pub auth: Auth,
    pub base_path: String,
    pub base_url: String,
    pub batch_path: String,
    pub description: String,
    pub discovery_version: String,
    pub documentation_link: String,
    #[serde(default)]
    pub icons: IndexMap<String, String>,
    pub id: String,
    pub kind: String,
    pub mtls_root_url: String,
    pub name: String,
    pub owner_domain: String,
    pub owner_name: String,
    #[serde(default)]
    pub parameters: IndexMap<String, Schema>,
    pub protocol: Protocol,
    #[serde(default)]
    pub resources: IndexMap<String, Resource>,
    pub revision: String,
    pub root_url: String,
    #[serde(default)]
    pub schemas: IndexMap<String, RootSchema>,
    pub service_path: String,
    pub title: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    pub oauth2: OAuth2,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2 {
    pub scopes: IndexMap<String, Scope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Rest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Post,
    Put,
    Delete,
    Get,
    Update,
    Patch,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Resource {
    pub methods: IndexMap<String, Endpoint>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Methods {
    pub delete: Option<Endpoint>,
    pub get: Option<Endpoint>,
    pub update: Option<Endpoint>,
    pub list: Option<Endpoint>,
    pub insert: Option<Endpoint>,
    pub patch: Option<Endpoint>,
    pub query: Option<Endpoint>,
    pub cancel: Option<Endpoint>,
    pub insert_all: Option<Endpoint>,
    pub get_service_account: Option<Endpoint>,
    pub get_query_results: Option<Endpoint>,
    pub set_iam_policy: Option<Endpoint>,
    pub get_iam_policy: Option<Endpoint>,
    pub test_iam_permissions: Option<Endpoint>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MediaUpload {
    pub accept: Vec<String>,
    pub protocols: MediaUploadProtocols,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MediaUploadProtocols {
    pub resumable: MediaUploadProtocolType,
    pub simple: MediaUploadProtocolType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MediaUploadProtocolType {
    pub path: String,
    pub multipart: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Endpoint {
    pub description: String,
    pub http_method: HttpMethod,
    pub id: String,
    pub parameter_order: Option<Vec<String>>,
    pub parameters: IndexMap<String, Schema>,
    pub request: Option<Schema>,
    pub media_upload: Option<MediaUpload>,
    pub path: String,
    pub response: Option<Schema>,
    pub supports_media_upload: Option<bool>,
    pub flat_path: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct RootSchema {
    pub id: String,
    #[serde(flatten)]
    inner: Schema,
}

impl RootSchema {
    pub fn into_id_schema_pair(self) -> (Cow<'static, str>, Schema) {
        (Cow::Owned(self.id), self.inner)
    }
}

impl std::ops::Deref for RootSchema {
    type Target = Schema;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for RootSchema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Annotations {
    pub required: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Schema {
    pub description: Option<String>,
    #[serde(default)]
    pub read_only: bool,
    pub annotations: Option<Annotations>,
    #[serde(default)]
    pub required: bool,
    #[serde(flatten)]
    pub schema_kind: SchemaKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum SchemaKind {
    Ref {
        #[serde(rename = "$ref")]
        refer: String,
    },
    Type {
        #[serde(flatten)]
        def: TypeDef,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TypeDef {
    String {
        #[serde(flatten)]
        type_def: Option<StringTypeDef>,
    },
    Array {
        items: Box<Schema>,
    },
    Boolean,
    Integer(Numeric),
    Number(Numeric),
    Object {
        #[serde(default)]
        properties: IndexMap<String, Schema>,
        #[serde(rename = "additionalProperties")]
        additional_properties: Option<Box<Schema>>,
    },
    Any,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Numeric {
    pub format: Format,
    pub minimum: Option<serde_json::Number>,
    pub maximum: Option<serde_json::Number>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum StringTypeDef {
    Enum(Enum),
    Numeric { format: Format },
    String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
    #[serde(rename = "enum")]
    pub enum_variants: Vec<String>,
    pub enum_descriptions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamLocation {
    Query,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Float,
    Double,
    Uint64,
    Int64,
    Uint32,
    Int32,
    Byte,
    #[serde(rename = "date-time")]
    DateTime,
    #[serde(rename = "google-datetime")]
    GoogleDateTime,
    #[serde(rename = "google-fieldmask")]
    GoogleFieldMask,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_json() -> anyhow::Result<()> {
        const RAW_JSON: &str = r#"{
            "description": "The status of the trial.",
            "enum": [
              "TRIAL_STATUS_UNSPECIFIED",
              "NOT_STARTED",
              "RUNNING",
              "SUCCEEDED",
              "FAILED",
              "INFEASIBLE",
              "STOPPED_EARLY"
            ],
            "enumDescriptions": [
              "Default value.",
              "Scheduled but not started.",
              "Running state.",
              "The trial succeeded.",
              "The trial failed.",
              "The trial is infeasible due to the invalid params.",
              "Trial stopped early because it's not promising."
            ],
            "type": "string"
        }"#;

        let schema: SchemaKind = serde_json::from_str(RAW_JSON)?;

        println!("{schema:#?}");
        Ok(())
    }

    #[test]
    fn test_input_data_change_json() -> anyhow::Result<()> {
        const RAW_JSON: &str = r#"{
            "description": "Details about the input data change insight.",
            "id": "InputDataChange",
            "properties": {
                "recordsReadDiffPercentage": {
                    "description": "Output only. Records read difference percentage compared to a previous run.",
                    "format": "float",
                    "readOnly": true,
                    "type": "number"
                }
            },
            "type": "object"
        }"#;

        let schema: SchemaKind = serde_json::from_str(RAW_JSON)?;

        println!("{schema:#?}");
        Ok(())
    }
}
