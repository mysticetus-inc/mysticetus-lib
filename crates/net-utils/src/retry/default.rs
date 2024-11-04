use http::{Request, Response};
use tonic::Code;

use super::backoff::{Classify, RetryClassify};

const GRPC_STATUS_HEADER_CODE: &str = "grpc-status";

/// Provided gRPC retry classifier.
///
/// Signals that the following cases should be retried:
///
///     - Any transport error (i.e before the server can even return a response)
///     - If the response headers has the gRPC status, it retries on Internal, Unavailable and
///       Unknown errors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrpcRetryClassifier;

impl<Req, Resp, E> RetryClassify<Request<Req>, Response<Resp>, E> for GrpcRetryClassifier {
    fn classify(&self, _req: &Request<Req>, result: Result<&Response<Resp>, &E>) -> Classify {
        let resp = match result {
            Ok(response) => response,
            Err(_) => return Classify::Retry,
        };

        if let Some(status) = resp.headers().get(GRPC_STATUS_HEADER_CODE) {
            match Code::from_bytes(status.as_bytes()) {
                Code::Unknown | Code::Internal | Code::Unavailable => {
                    return Classify::Retry;
                }
                _ => (),
            }
        }

        Classify::DontRetry
    }
}
