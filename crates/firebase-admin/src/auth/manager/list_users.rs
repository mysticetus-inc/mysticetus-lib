use std::num::NonZeroUsize;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio_util::sync::ReusableBoxFuture;

use super::AuthManager;
use crate::auth::UserInfo;

pin_project_lite::pin_project! {
    pub struct ListUsersStream {
        url: reqwest::Url,
        page_size: NonZeroUsize,
        #[pin]
        fut: ReusableBoxFuture<'static, (AuthManager, crate::Result<(Option<String>, Vec<UserInfo>)>)>,
        state: State,
    }
}

impl ListUsersStream {
    pub(super) fn new(manager: AuthManager, page_size: usize) -> Self {
        let page_size = NonZeroUsize::new(page_size).unwrap_or(NonZeroUsize::MIN);
        let mut url = (*manager.base_url).clone();

        {
            url.path_segments_mut()
                .expect("can be a base")
                .push("accounts:batchGet");
        }

        let fut = ReusableBoxFuture::new(make_request(manager, url.clone(), page_size, None));

        Self {
            url,
            page_size,
            state: State::Requesting,
            fut,
        }
    }

    pub async fn next(&mut self) -> crate::Result<Option<Vec<UserInfo>>> {
        let mut pinned = Pin::new(self);
        std::future::poll_fn(|cx| pinned.as_mut().poll_next(cx))
            .await
            .transpose()
    }

    pub async fn collect(mut self) -> crate::Result<Vec<UserInfo>> {
        let mut users = match self.next().await? {
            None => return Ok(vec![]),
            Some(users) => users,
        };

        // handle getting a single page specially,
        match self.next().await? {
            None => return Ok(users),
            Some(mut more_users) => users.append(&mut more_users),
        }

        // drain any more pages
        while let Some(mut more_users) = self.next().await? {
            users.append(&mut more_users);
        }

        Ok(users)
    }
}

enum State {
    Requesting,
    Done,
}

impl Stream for ListUsersStream {
    type Item = crate::Result<Vec<UserInfo>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.state {
            State::Done => Poll::Ready(None),
            State::Requesting => {
                let (manager, result) = std::task::ready!(this.fut.as_mut().poll(cx));

                match result {
                    Err(error) => {
                        *this.state = State::Done;
                        Poll::Ready(Some(Err(error)))
                    }
                    Ok((Some(next_page_token), users)) => {
                        // kick off the next request
                        this.fut.get_mut().set(make_request(
                            manager,
                            this.url.clone(),
                            *this.page_size,
                            Some(next_page_token),
                        ));
                        Poll::Ready(Some(Ok(users)))
                    }
                    Ok((None, users)) => {
                        *this.state = State::Done;
                        Poll::Ready(Some(Ok(users)))
                    }
                }
            }
        }
    }
}

// make requests through the same, non-generic function, that way
// the future should have a consistent layout, letting ReusableBoxFuture
// reuse the memory
async fn make_request(
    manager: AuthManager,
    url: reqwest::Url,
    page_size: NonZeroUsize,
    next_page_token: Option<String>,
) -> (AuthManager, crate::Result<(Option<String>, Vec<UserInfo>)>) {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ListUsersPage {
        #[serde(default)]
        users: Vec<UserInfo>,
        next_page_token: Option<String>,
    }

    // use an inner future for error handling
    let future = async {
        let response = manager
            .request(url, reqwest::Method::GET, move |mut builder| {
                builder = builder.query(&[("maxResults", page_size.get())]);

                if let Some(token) = next_page_token {
                    builder = builder.query(&[("nextPageToken", token)]);
                }

                builder
            })
            .await?;

        let ListUsersPage {
            users,
            next_page_token,
        } = super::parse_json_response(response).await?;

        Ok((next_page_token, users))
    };

    let result = future.await;

    (manager, result)
}
