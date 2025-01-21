use std::num::NonZeroU64;

use bigquery_resources_rs::query::{
    QueryParameter, QueryParameterType, QueryParameterValue, QueryRequest, QueryResponse,
};
use bigquery_resources_rs::table::FieldType;

use crate::BigQueryClient;

pub struct QueryBuilder<S = Box<str>> {
    client: BigQueryClient,
    request: QueryRequest<S>,
}

impl<S> QueryBuilder<S> {
    pub(crate) fn new(client: BigQueryClient, request: QueryRequest<S>) -> Self {
        Self { client, request }
    }

    pub fn limit(mut self, limit: NonZeroU64) -> Self {
        self.request.max_results = Some(limit);
        self
    }

    pub fn number_param(
        &mut self,
        name: impl Into<S>,
        param: impl Into<serde_json::Number>,
    ) -> &mut Self {
        self.request.parameter_mode = Some(bigquery_resources_rs::query::ParameterMode::Named);

        let number: serde_json::Number = param.into();

        let ty = if number.is_f64() {
            FieldType::Float
        } else {
            FieldType::Integer
        };

        self.request.query_parameters.push(QueryParameter {
            name: Some(name.into()),
            parameter_type: QueryParameterType::Scalar(ty),
            parameter_value: QueryParameterValue::Scalar(number.into()),
        });

        self
    }

    pub fn string_param(&mut self, name: impl Into<S>, param: impl Into<String>) -> &mut Self {
        self.request.parameter_mode = Some(bigquery_resources_rs::query::ParameterMode::Named);
        self.request.query_parameters.push(QueryParameter {
            name: Some(name.into()),
            parameter_type: QueryParameterType::Scalar(FieldType::String),
            parameter_value: QueryParameterValue::Scalar(serde_json::Value::String(param.into())),
        });
        self
    }

    pub async fn execute<Row, S2>(&self) -> crate::Result<QueryResponse<Row, S2>>
    where
        S: serde::Serialize + std::fmt::Debug,
        S2: AsRef<str> + serde::de::DeserializeOwned,
        Row: serde::de::DeserializeOwned,
        QueryResponse<Row, S2>: std::fmt::Debug,
    {
        call_query(&self.client, &self.request).await
    }
}

async fn call_query<Row, S1, S2>(
    client: &BigQueryClient,
    request: &QueryRequest<S1>,
) -> crate::Result<QueryResponse<Row, S2>>
where
    S1: serde::Serialize + std::fmt::Debug,
    S2: AsRef<str> + serde::de::DeserializeOwned,
    Row: serde::de::DeserializeOwned,
    QueryResponse<Row, S2>: std::fmt::Debug,
{
    let url = client.inner.make_url(["queries"]);

    let resp = client
        .inner
        .request(reqwest::Method::POST, url)
        .await?
        .json(request)
        .send()
        .await?;

    crate::client::handle_json_response(resp).await
}

#[cfg(test)]
mod query_tests {
    use bigquery_resources_rs::query;
    use bigquery_resources_rs::query::QueryString;

    use super::*;

    #[tokio::test]
    async fn test_query() -> crate::Result<()> {
        let dataset =
            BigQueryClient::new("mysticetus-boem", gcp_auth_channel::Scope::BigQueryReadOnly)
                .await?
                .into_dataset::<&'static str>("main");

        const QUERY: QueryString = query!("SELECT * FROM current_projects LIMIT 10");

        let q: QueryBuilder = dataset.query(QUERY);

        let response: QueryResponse<serde_json::Value> = q.execute().await?;

        println!("{response:#?}");

        Ok(())
    }
}
