use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::Stream;
use protos::spanner::batch_write_request::MutationGroup;
use protos::spanner::{self};

pub struct BatchWriteErrors {
    mutation_groups: Vec<MutationGroup>,
    /// pairs containing:
    /// - mutation group indices, and
    /// - a usize that maps to the corresponding status in 'statuses'
    errors: Vec<(Vec<i32>, usize)>,
    statuses: Vec<protos::rpc::Status>,
}

pin_project_lite::pin_project! {
    #[project = BatchWriteProj]
    pub struct BatchWrite {
        mutation_groups: Vec<MutationGroup>,
        errors: Vec<(Vec<i32>, usize)>,
        statuses: Vec<protos::rpc::Status>,
        completed: usize,
        #[pin]
        response: tonic::Streaming<spanner::BatchWriteResponse>,
    }
}

impl BatchWriteProj<'_> {
    fn is_complete(&self) -> bool {
        *self.completed >= self.mutation_groups.len()
    }

    fn into_errors(&mut self) -> BatchWriteErrors {
        todo!()
    }

    fn handle_response(&mut self, resp: spanner::BatchWriteResponse) {
        let status = resp.status.expect("google didnt send back a status?");
        if status.code != tonic::Code::Ok as i32 && !resp.indexes.is_empty() {
            // statuses.len will map to the index that the status will end up at
            self.errors.push((resp.indexes, self.statuses.len()));
            self.statuses.push(status);
        } else {
            *self.completed += resp.indexes.len();
        }
    }
}

impl Stream for BatchWrite {
    type Item = crate::Result<BatchWriteErrors>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match ready!(this.response.as_mut().poll_next(cx)) {
                Some(Ok(resp)) => {
                    this.handle_response(resp);
                }
                Some(Err(error)) => return Poll::Ready(Some(Err(error.into()))),
                None if this.is_complete() => return Poll::Ready(None),
                None => todo!("resend request with the mutation groups that errored out"),
            }
        }
    }
}
