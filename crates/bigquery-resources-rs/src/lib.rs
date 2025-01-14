use std::borrow::Borrow;
use std::fmt;

pub mod dataset;
pub mod job;
pub mod query;
pub mod table;
pub mod table_data;
mod util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableReference<S = Box<str>> {
    pub project_id: S,
    pub dataset_id: S,
    pub table_id: S,
}

impl<S> TableReference<S> {
    #[inline]
    pub fn as_deref(&self) -> TableReference<&S::Target>
    where
        S: std::ops::Deref,
    {
        TableReference {
            project_id: self.project_id.deref(),
            dataset_id: self.dataset_id.deref(),
            table_id: self.table_id.deref(),
        }
    }

    #[inline]
    pub const fn dataset_reference(&self) -> DatasetReference<&S> {
        DatasetReference {
            project_id: &self.project_id,
            dataset_id: &self.dataset_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetReference<S = Box<str>> {
    pub project_id: S,
    pub dataset_id: S,
}

impl<S> DatasetReference<S> {
    #[inline]
    pub fn as_deref(&self) -> DatasetReference<&S::Target>
    where
        S: std::ops::Deref,
    {
        DatasetReference {
            project_id: self.project_id.deref(),
            dataset_id: self.dataset_id.deref(),
        }
    }

    #[inline]
    pub fn as_table<'a>(&'a self, table_id: &'a impl Borrow<S>) -> TableReference<&'a S> {
        TableReference {
            project_id: &self.project_id,
            dataset_id: &self.dataset_id,
            table_id: table_id.borrow(),
        }
    }

    #[inline]
    pub fn into_table(self, table_id: S) -> TableReference<S> {
        TableReference {
            project_id: self.project_id,
            dataset_id: self.dataset_id,
            table_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto<S = Box<str>> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<S>,
    pub message: S,
}

impl<S: fmt::Display> fmt::Display for ErrorProto<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            Some(ref reason) => write!(f, "{}: {reason}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl<S: AsRef<str>> ErrorProto<S> {
    pub fn is_not_found(&self) -> bool {
        self.reason
            .as_ref()
            .is_some_and(|reason| reason.as_ref() == "notFound")
    }
}
