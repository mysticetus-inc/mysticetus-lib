use bytes::Bytes;
use net_utils::backoff::Backoff;
use net_utils::bidi2::{RequestSink, build_pair};
use protos::firestore::listen_request::TargetChange as ListenTargetChange;
use protos::firestore::listen_response::ResponseType;
// use net_utils::open_close::{self, OpenClose};
use protos::firestore::target::{ResumeType, TargetType};
use protos::firestore::target_change::TargetChangeType;
use protos::firestore::{
    Document, DocumentChange, ExistenceFilter, ListenRequest, ListenResponse, Target, TargetChange,
};
use timestamp::{Duration, Timestamp};

// single target id, similar to the c# and python client
// (though we use 'rs' instead of 'c#' or 'py')
const TARGET_ID: i32 = 0x7273;

// default backoff, copying params from the c# client. Since this is
// continually reused, we set a huge number of backoff attempts so we should never actually
// hit the max retry case.
const BACKOFF: Backoff = Backoff::builder()
    .base_delay(Duration::from_seconds(1))
    .max_timeout(Duration::from_seconds(30))
    .max_retries(u32::MAX)
    .build_local();

use super::TargetId;
use crate::Firestore;

#[pin_project::pin_project(project = ListenerProj)]
pub struct Listener {
    state: ListenerState,
    #[pin]
    stream: ListenerStreamState,
}

struct ListenerState {
    firestore: Firestore,
    backoff: Backoff,
    current: bool,
    change_map: fxhash::FxHashMap<Box<str>, Document>,
    resume_bytes: Option<Bytes>,
}

struct TargetState {
    id: TargetId,
}

#[pin_project::pin_project(project = ListenerStreamStateProj)]
enum ListenerStreamState {
    Alive {
        #[pin]
        stream: ListenerStream,
    },
    Pending,
    Closed,
}

#[pin_project::pin_project(project = ListenerStreamProj)]
struct ListenerStream {
    sink: RequestSink<ListenRequest>,
    #[pin]
    stream: tonic::Streaming<ListenResponse>,
}

enum ListenResponseResult {
    Continue,
    ResetStream,
}

fn has_valid_target_ids(target_ids: &[i32]) -> bool {
    target_ids.is_empty() || target_ids.contains(&TARGET_ID)
}

pub struct ChangeSet {
    deletes: Vec<Box<str>>,
    adds: Vec<Box<str>>,
    updates: Vec<Box<str>>,
}

impl ListenerState {
    fn handle_response(&mut self, response: ListenResponse) -> crate::Result<ListenResponseResult> {
        // we don't want to unwrap, but we do want to error out of the response is null/invalid.
        // this matches the c# client WatchState behavior when encountering an unknown variant.

        let response_type = response.response_type.ok_or(crate::Error::Internal(
            "ListenResponse.response_type is invalid",
        ))?;

        self.handle_response_type(response_type)
    }

    fn handle_response_type(
        &mut self,
        response: ResponseType,
    ) -> crate::Result<ListenResponseResult> {
        match response {
            ResponseType::TargetChange(change) => self.handle_target_change(change),
            ResponseType::Filter(filter) => self.handle_filter(filter),
            ResponseType::DocumentChange(change) => self.handle_doc_change(change),
            ResponseType::DocumentDelete(del) => self.handle_doc_remove_delete(del.document),
            ResponseType::DocumentRemove(rem) => self.handle_doc_remove_delete(rem.document),
        }
    }

    fn handle_filter(&mut self, filter: ExistenceFilter) -> crate::Result<ListenResponseResult> {
        todo!("{filter:#?}")
    }

    fn handle_target_change(
        &mut self,
        change: TargetChange,
    ) -> crate::Result<ListenResponseResult> {
        let has_resume_token = !change.resume_token.is_empty();

        match change.target_change_type() {
            TargetChangeType::NoChange => {
                // This means everything is up-to-date, so emit the current set of docs as a
                // snapshot, if there were changes.
                if let Some(read_time) = change.read_time
                    && change.target_ids.is_empty()
                    && self.current
                {
                    self.push_snapshot(
                        read_time.into(),
                        crate::util::none_if_empty(change.resume_token),
                    )?;
                }
            }
            TargetChangeType::Add => debug_assert!(has_valid_target_ids(&change.target_ids)),
            // according to the c# client, this only happens when the server aborts in an
            // unrecoverable way, so we should throw
            TargetChangeType::Remove => {
                return match change.cause {
                    Some(status) => Err(crate::Error::from(status)),
                    None => Err(crate::Error::RpcError {
                        code: tonic::Code::Unknown,
                        message: "Unknown cause".to_owned(),
                        details: None,
                    }),
                };
            }
            TargetChangeType::Current => self.current = true,
            TargetChangeType::Reset => self.reset(),
        }

        // if the stream is 'healthy', we should reset the backoff
        if has_resume_token && has_valid_target_ids(&change.target_ids) {
            self.backoff = BACKOFF;
        }

        Ok(ListenResponseResult::Continue)
    }

    fn on_stream_init(&mut self, stream_completed_or_errored: bool) {
        self.current = false;
        if stream_completed_or_errored {
            self.change_map.clear();
        }
    }

    fn reset(&mut self) {
        self.change_map.clear();
        self.resume_bytes = None;
        self.current = false;
    }

    fn handle_doc_change(&mut self, change: DocumentChange) -> crate::Result<ListenResponseResult> {
        let changed = change.target_ids.contains(&TARGET_ID);
        let removed = change.removed_target_ids.contains(&TARGET_ID);

        if changed && removed {
            return Err(crate::Error::RpcError {
                code: tonic::Code::Internal,
                message: "server both removed and changed a document".to_owned(),
                details: None,
            });
        } else if !changed && !removed {
            // This is probably an error in the server, but we can follow protocol by just ignoring
            // this response.
            return Ok(ListenResponseResult::Continue);
        }

        let document = change.document.ok_or(crate::Error::Internal(
            "document was changed but not included in DocumentChange",
        ))?;

        if changed {
            self.change_map
                .insert(document.name.as_str().into(), document);
        } else {
            self.change_map.remove(document.name.as_str());
        }

        Ok(ListenResponseResult::Continue)
    }

    fn handle_doc_remove_delete(&mut self, doc: String) -> crate::Result<ListenResponseResult> {
        self.change_map.remove(doc.as_str());
        Ok(ListenResponseResult::Continue)
    }

    fn push_snapshot(
        &mut self,
        read_time: Timestamp,
        resume_token: Option<Bytes>,
    ) -> crate::Result<()> {
        todo!()
    }
}

impl Listener {
    async fn next_message(&mut self) -> crate::Result<Option<ListenResponse>> {
        // inner method that doesn't handle error cases or restarting a dead stream
        async fn next_message_inner(
            state: &mut ListenerState,
            stream_state: &mut ListenerStreamState,
            stream_complete_or_error: &mut bool,
        ) -> crate::Result<Option<ListenResponse>> {
            loop {
                match stream_state {
                    ListenerStreamState::Alive { stream } => {
                        return stream.stream.message().await.map_err(crate::Error::from);
                    }
                    ListenerStreamState::Closed => return Ok(None),
                    ListenerStreamState::Pending => {
                        let backoff = state
                            .backoff
                            .backoff_once()
                            .expect("overflowed u32::MAX backoffs");

                        // only back off if we're not on the very first try
                        if backoff.on_retry() > 1 {
                            backoff.await;
                        }

                        let (sink, request_stream) = build_pair();

                        let stream = state
                            .firestore
                            .client
                            .get()
                            .listen(request_stream)
                            .await?
                            .into_inner();

                        state.on_stream_init(*stream_complete_or_error);

                        *stream_state = ListenerStreamState::Alive {
                            stream: ListenerStream { sink, stream },
                        };
                    }
                }
            }
        }

        let mut stream_complete_or_error = false;

        loop {
            match next_message_inner(
                &mut self.state,
                &mut self.stream,
                &mut stream_complete_or_error,
            )
            .await
            {
                Ok(Some(message)) => return Ok(Some(message)),
                // we wont restart a manually closed listener
                Ok(None) if matches!(self.stream, ListenerStreamState::Closed) => return Ok(None),
                // if we're None and not closed, the stream itself is exhausted, so we need to
                // restart it.
                Ok(None) => {
                    self.stream = ListenerStreamState::Pending;
                    stream_complete_or_error = true;
                }
                Err(error) => match error.rpc_code() {
                    Some(code) if crate::error::is_transient_error(code) => {
                        self.stream = ListenerStreamState::Pending;
                        stream_complete_or_error = true;

                        // do some extra backing off if this is the error, as per the c# client
                        if matches!(code, tonic::Code::ResourceExhausted) {
                            self.state
                                .backoff
                                .backoff_once()
                                .expect("overflowed u32::MAX retries")
                                .await;
                        }
                    }
                    _ => return Err(error),
                },
            }
        }
    }

    async fn add_target(
        &mut self,
        target: TargetType,
        once: bool,
        resume_type: Option<ResumeType>,
    ) -> crate::Result<TargetId> {
        let target_id = TargetId::next_id();

        let target = Target {
            target_id: target_id.get(),
            once,
            resume_type,
            expected_count: None,
            target_type: Some(target),
        };

        self.send_message(ListenTargetChange::AddTarget(target))
            .await?;

        Ok(target_id)
    }

    async fn send_message(&mut self, change: ListenTargetChange) -> crate::Result<()> {
        let message = ListenRequest {
            database: self.state.firestore.qualified_db_path().to_owned(),
            labels: Default::default(),
            target_change: Some(change),
        };

        self.get_or_start_stream()
            .await?
            .ok_or(crate::Error::ListenerClosed)?
            .1
            .sink
            .send(message)
            .map_err(|_| crate::Error::ListenerClosed)
    }

    pub(crate) fn new(firestore: Firestore) -> Self {
        Self {
            state: ListenerState {
                current: false,
                firestore,
                resume_bytes: None,
                change_map: fxhash::FxHashMap::with_capacity_and_hasher(
                    16,
                    fxhash::FxBuildHasher::default(),
                ),
                backoff: BACKOFF,
            },
            stream: ListenerStreamState::Pending,
        }
    }
}

pub struct Change {}
