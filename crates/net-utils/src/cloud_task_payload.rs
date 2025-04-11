use axum::extract::FromRequest;
use axum::response::IntoResponse;
use bytes::Bytes;
use http::StatusCode;

pub struct CloudTaskPayload<P>(pub P);

impl<S, P> FromRequest<S> for CloudTaskPayload<P>
where
    P: serde::de::DeserializeOwned,
{
    type Rejection = Rejection;

    fn from_request(
        req: axum::extract::Request,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let bytes = Bytes::from_request(req, &())
                .await
                .map_err(Rejection::Buffering)?;

            let payload = deserialize_cloud_task_payload(&bytes)?;
            Ok(Self(payload))
        }
    }
}

#[inline]
fn deserialize_cloud_task_payload<P>(bytes: &[u8]) -> Result<P, Rejection>
where
    P: serde::de::DeserializeOwned,
{
    path_aware_serde::json::deserialize_slice(bytes).map_err(Rejection::Json)
}

#[derive(Debug, thiserror::Error)]
pub enum Rejection {
    #[error(transparent)]
    Json(#[from] path_aware_serde::Error<serde_json::Error>),
    #[error(transparent)]
    Buffering(<Bytes as FromRequest<()>>::Rejection),
}

impl IntoResponse for Rejection {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::Buffering(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Json(_) => StatusCode::BAD_REQUEST,
        };

        (status, self.to_string()).into_response()
    }
}
