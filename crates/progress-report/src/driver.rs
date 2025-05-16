use firestore_rs::{DocFields, DocumentRef, PathComponent, Reference};
use net_utils::backoff::Backoff;
use timestamp::Timestamp;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Drives writing progress reports to firestore in a background task.
pub struct Driver<CollecId: PathComponent> {
    incoming: mpsc::UnboundedSender<Incoming>,
    #[allow(unused)]
    handle: JoinHandle<()>,
    doc_ref: DocumentRef<CollecId, Uuid>,
}

struct Incoming {
    update_timestamp: Timestamp,
    fields: DocFields,
    /// None if the caller doesn't care about the result of updating the firestore doc,
    /// Some if they do.
    result_tx: Option<oneshot::Sender<firestore_rs::Result<(Box<Reference>, Timestamp)>>>,
}

impl<CollecId: PathComponent + Clone + Sync + Send + 'static> Driver<CollecId> {
    pub fn new(doc_ref: DocumentRef<CollecId, Uuid>) -> Self {
        let (incoming, rx) = mpsc::unbounded_channel();
        Self {
            doc_ref: doc_ref.clone(),
            handle: tokio::spawn(driver_loop(doc_ref, rx)),
            incoming,
        }
    }

    fn send_inner(&mut self, incoming: Incoming) {
        // if the handle is closed, the task panic'd for some reason, so restart it.
        if self.incoming.is_closed() {
            *self = Self::new(self.doc_ref.clone());
        }

        self.incoming
            .send(incoming)
            .expect("we restarted the handle if it crashed, so this should be alive");
    }

    pub fn send_recv_result(
        &mut self,
        update_timestamp: Timestamp,
        fields: DocFields,
    ) -> oneshot::Receiver<firestore_rs::Result<(Box<Reference>, Timestamp)>> {
        let (result_tx, result_rx) = oneshot::channel();

        self.send_inner(Incoming {
            update_timestamp,
            fields,
            result_tx: Some(result_tx),
        });

        result_rx
    }

    pub fn send(&mut self, update_timestamp: Timestamp, fields: DocFields) {
        self.send_inner(Incoming {
            update_timestamp,
            fields,
            result_tx: None,
        });
    }
}

async fn set<CollecId: PathComponent>(
    doc_ref: &DocumentRef<CollecId, Uuid>,
    fields: DocFields,
) -> firestore_rs::Result<(Box<Reference>, Timestamp)> {
    async fn try_set<CollecId: PathComponent>(
        doc_ref: &DocumentRef<CollecId, Uuid>,
        fields: DocFields,
    ) -> firestore_rs::Result<(Box<Reference>, Timestamp)> {
        let write = doc_ref
            .set_serialized::<serde::de::IgnoredAny>(fields)
            .await?;

        let update_time = write.update_time.ok_or(firestore_rs::Error::Internal(
            "expected Doc.update_time to be Some on a write",
        ))?;

        Ok((write.reference, update_time))
    }

    let mut backoff = Backoff::default();

    loop {
        match try_set(doc_ref, fields.clone()).await {
            Ok((refer, timestamp)) => break Ok((refer, timestamp)),
            Err(error) if error.is_transient_error() => match backoff.backoff_once() {
                Some(backoff) => backoff.await,
                None => break Err(error),
            },
            Err(error) => break Err(error),
        }
    }
}

async fn set_incoming<CollecId: PathComponent>(
    doc_ref: &DocumentRef<CollecId, Uuid>,
    incoming: Incoming,
) {
    let final_result = set(doc_ref, incoming.fields).await;

    match incoming.result_tx {
        Some(tx) => {
            if let Err(result) = tx.send(final_result) {
                log_result(doc_ref, result)
            }
        }
        None => log_result(doc_ref, final_result),
    }
}

fn log_result<CollecId: PathComponent>(
    doc_ref: &DocumentRef<CollecId, Uuid>,
    result: firestore_rs::Result<(Box<Reference>, Timestamp)>,
) {
    match result {
        Ok((reference, timestamp)) => tracing::debug!(
            message = "updated progress document",
            ?reference,
            %timestamp
        ),
        Err(error) if error.is_transient_error() => tracing::error!(
            message="failed to update progress document after retrying",
            %error,
            reference=?doc_ref.reference()
        ),
        Err(error) => tracing::error!(
            message="failed to update progress document",
            %error,
            reference=?doc_ref.reference()
        ),
    }
}

async fn driver_loop<CollecId: PathComponent + Sync + Send + 'static>(
    doc_ref: DocumentRef<CollecId, Uuid>,
    mut rx: mpsc::UnboundedReceiver<Incoming>,
) {
    // get the first message
    let Some(incoming) = rx.recv().await else {
        return;
    };

    let mut current_timestamp = incoming.update_timestamp;
    let mut current_set_future = std::pin::pin!(set_incoming(&doc_ref, incoming));

    loop {
        // wait for both to complete, since we need to wait for the prior request to complete before
        // we can start the next one. Ideally we wouldn't need to, but dropping the current future
        // to start the next request doesn't always work as expected, hyper does work in background
        // tokio tasks that we can't stop. This can lead to document writes happening out of order,
        // in unpredictable ways.
        let (_, mut next) = tokio::join!(&mut current_set_future, rx.recv());

        loop {
            let Some(incoming) = next.take() else {
                return;
            };

            // only set the document if the timestamp is after/identical
            // the one we just finished writing.
            if current_timestamp <= incoming.update_timestamp {
                current_timestamp = incoming.update_timestamp;
                current_set_future.set(set_incoming(&doc_ref, incoming));
                break;
            }

            // if 'incoming' was before, we need to get another incoming before we can poll
            // current_set_future again, since it's already done.
            next = rx.recv().await;
        }
    }
}
