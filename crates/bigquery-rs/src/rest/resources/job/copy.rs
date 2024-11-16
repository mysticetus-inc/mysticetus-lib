// 'empty' enum to indicate this is a TODO item
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobConfigurationTableCopy<S> {
    #[doc(hidden)]
    __ToDo(std::marker::PhantomData<S>),
}
