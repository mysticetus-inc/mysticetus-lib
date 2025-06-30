use futures::{Stream, stream};
use protos::firestore::ListCollectionIdsRequest;

use crate::client::FirestoreClient;

const DEFAULT_PAGINATION_SIZE: i32 = 1000;

enum StreamState<A> {
    Continue(A),
    Finished,
}

impl<A> StreamState<A> {
    fn into_inner(self) -> Option<A> {
        match self {
            Self::Continue(inner) => Some(inner),
            Self::Finished => None,
        }
    }
}

pub(crate) fn list_collection_ids<'a, P>(
    client: &'a mut FirestoreClient,
    parent: P,
) -> impl Stream<Item = crate::Result<Vec<String>>> + Send + 'a
where
    P: AsRef<str> + Send + 'a,
{
    let init_request = ListCollectionIdsRequest {
        parent: parent.as_ref().to_owned(),
        page_size: DEFAULT_PAGINATION_SIZE,
        page_token: String::new(),
        consistency_selector: None,
    };

    let init_state = StreamState::Continue((client, parent, init_request));

    stream::unfold(init_state, move |state| async move {
        let (client, parent, list_request) = state.into_inner()?;

        let resp = match client.get().list_collection_ids(list_request).await {
            Ok(resp) => resp.into_inner(),
            Err(err) => return Some((Err(err.into()), StreamState::Finished)),
        };

        if resp.next_page_token.is_empty() {
            Some((Ok(resp.collection_ids), StreamState::Finished))
        } else {
            let next_request = ListCollectionIdsRequest {
                parent: parent.as_ref().to_owned(),
                page_size: DEFAULT_PAGINATION_SIZE,
                page_token: resp.next_page_token,
                consistency_selector: None,
            };

            let next_state = StreamState::Continue((client, parent, next_request));

            Some((Ok(resp.collection_ids), next_state))
        }
    })
}

/*
pub(crate) fn database_path_from_resource_path<S>(resource_path: S) -> crate::Result<String>
where
    S: AsRef<str>,
{
    let resource = resource_path.as_ref();
    // Skip the first, then step by 2, that way we only extract the 2nd and 4th components
    let mut path_iter = resource.split('/').skip(1).step_by(2);
    let project_id = extract_component!(path_iter);
    let database_id = extract_component!(path_iter);

    Ok(format!("projects/{project_id}/databases/{database_id}"))
}
*/
