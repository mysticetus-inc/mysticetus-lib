pub struct RetryOnStatusPolicy<C> {
    classifier: C,
}

pub trait StatusClassifier {
    fn should_retry(&self, status: tonic::Status) -> bool;
}
