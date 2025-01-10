use super::ErrorProto;

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableDataInsertAllResponse<S = Box<str>> {
    /// An array of errors for rows that were not inserted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub insert_errors: Vec<InsertErrors<S>>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertErrors<S> {
    /// The index of the row that error applies to.
    pub index: usize,
    /// Error information for the row indicated by the index property.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorProto<S>>,
}
