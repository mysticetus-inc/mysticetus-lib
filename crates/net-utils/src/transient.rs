/// Classifies whether a grpc error is transient (i.e should be retried) or not.
pub trait IsTransient {
    fn is_transient(&self, error: &tonic::Status) -> bool;
}

impl<S: std::hash::BuildHasher> IsTransient for std::collections::HashSet<tonic::Code, S> {
    fn is_transient(&self, error: &tonic::Status) -> bool {
        self.contains(&error.code())
    }
}

/// Default classifier for transient errors. Codes that are considered transient were pulled from
/// one of the google grpc client libraries, [here].
///
/// [here]: <https://github.com/googleapis/google-cloud-dotnet/blob/a640e146b40b19a87318deace4f7bfd19e4b860f/apis/Google.Cloud.Firestore/Google.Cloud.Firestore/WatchStream.cs#L29-L39>
pub struct DefaultTransientErrors;

impl IsTransient for DefaultTransientErrors {
    fn is_transient(&self, error: &tonic::Status) -> bool {
        use tonic::Code::{
            Aborted, Cancelled, DeadlineExceeded, Internal, ResourceExhausted, Unauthenticated,
            Unavailable, Unknown,
        };

        match error.code() {
            Aborted | Cancelled | Unknown | DeadlineExceeded | ResourceExhausted | Internal
            | Unavailable | Unauthenticated => true,
            _ => false,
        }
    }
}
