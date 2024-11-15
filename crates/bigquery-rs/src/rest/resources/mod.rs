pub mod dataset;
pub mod job;
pub mod table;
pub mod table_data;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableReference<S = Box<str>> {
    pub project_id: S,
    pub dataset_id: S,
    pub table_id: S,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto<S = Box<str>> {
    pub reason: S,
    pub location: S,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<S>,
    pub message: S,
}