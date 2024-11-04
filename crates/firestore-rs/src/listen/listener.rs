use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::Stream;
use net_utils::open_close::{self, OpenClose};
use protos::firestore::target::query_target::QueryType;
use protos::firestore::target::{DocumentsTarget, QueryTarget, TargetType};
use protos::firestore::{ListenRequest, ListenResponse, StructuredQuery, Target, listen_request};

use super::ListenerId;
use crate::client::FirestoreClient;

#[pin_project::pin_project(project = ListenerProj)]
#[must_use = "[`Listener::close`] must be called to properly close the gRPC connection"]
pub struct Listener {
    id: ListenerId,
    database: String,
    #[pin]
    handle: OpenClose<ListenRequest, ListenResponse>,
}

impl Listener {
    pub(crate) async fn init_query<D, P>(
        client: &mut FirestoreClient,
        database: D,
        parent: P,
        query: StructuredQuery,
    ) -> crate::Result<Self>
    where
        D: Into<String>,
        P: Into<String>,
    {
        let target = TargetType::Query(QueryTarget {
            parent: parent.into(),
            query_type: Some(QueryType::StructuredQuery(query)),
        });

        Self::init(client, database.into(), target).await
    }

    pub(crate) async fn init_docs<D>(
        client: &mut FirestoreClient,
        database: D,
        documents: Vec<String>,
    ) -> crate::Result<Self>
    where
        D: Into<String>,
    {
        let target = TargetType::Documents(DocumentsTarget { documents });

        Self::init(client, database.into(), target).await
    }

    #[allow(dead_code)]
    pub(crate) async fn message(&mut self) -> crate::Result<Option<ListenResponse>> {
        self.handle.message().await.map_err(crate::Error::from)
    }

    pub(crate) async fn init(
        client: &mut FirestoreClient,
        database: String,
        target: TargetType,
    ) -> crate::Result<Self> {
        let id = ListenerId::gen_rand();

        let (req_stream, partially_init) = open_close::build_parts(ListenRequest {
            database: database.clone(),
            target_change: Some(listen_request::TargetChange::AddTarget(Target {
                expected_count: None,
                once: true,
                resume_type: None,
                target_type: Some(target),
                target_id: id.get(),
            })),
            labels: Default::default(),
        });

        let handle = partially_init
            .try_initialize(client.get().listen(req_stream))
            .await?;

        Ok(Self {
            id,
            database,
            handle,
        })
    }

    pub async fn close(mut self) -> crate::Result<()> {
        let req = ListenRequest {
            database: self.database,
            target_change: Some(listen_request::TargetChange::RemoveTarget(self.id.get())),
            labels: Default::default(),
        };

        if self.handle.close(req).is_err() {
            panic!("should only be closed once")
        }

        while let Some(res) = self.handle.message().await? {
            println!("close message: {res:#?}");
        }

        Ok(())
    }
}

pub struct Change {}

impl Stream for Listener {
    type Item = crate::Result<Change>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        let _event = match ready!(this.handle.poll_next(cx)) {
            Some(Ok(event)) => event,
            Some(Err(error)) => return Poll::Ready(Some(Err(error.into()))),
            None => return Poll::Ready(None),
        };

        Poll::Ready(None)
    }
}
