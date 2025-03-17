use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UserInfo {
    #[serde(flatten)]
    inner: BTreeMap<Box<str>, serde_json::Value>,
}
