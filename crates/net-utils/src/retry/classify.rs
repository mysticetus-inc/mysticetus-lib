use crate::http_svc::HttpResponse;

pub trait ClassifyResponse<Resp, Error>: Clone {
    fn should_retry(&self, result: &Result<Resp, Error>) -> ShouldRetry;
}

pub enum ShouldRetry {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultClassify;

impl<Resp: HttpResponse, Error> ClassifyResponse<Resp, Error> for DefaultClassify {
    fn should_retry(&self, result: &Result<Resp, Error>) -> ShouldRetry {
        match result {
            Ok(resp) if default_should_retry_response(resp) => ShouldRetry::Yes,
            _ => ShouldRetry::No,
        }
    }
}

fn default_should_retry_response(resp: &impl HttpResponse) -> bool {
    use tonic::Code::{Internal, ResourceExhausted, Unavailable};

    if resp.status().is_server_error() {
        return true;
    }

    if resp.status().is_client_error() {
        return false;
    }

    match resp.grpc_status() {
        Some(Internal | ResourceExhausted | Unavailable) => true,
        _ => false,
    }
}
