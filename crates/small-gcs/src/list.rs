use std::borrow::Cow;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::Stream;
use pin_project_lite::pin_project;
use reqwest::{RequestBuilder, header};
use tokio_util::sync::ReusableBoxFuture;

use crate::client::Client;
use crate::query_param::{Param, StringParam};
use crate::{Error, Object};

const DEFAULT_MAX_PER_PAGE: usize = 1000;

pub struct ListBuilder<'a> {
    shared: Cow<'a, Client>,
    builder: RequestBuilder,
}

impl<'a> ListBuilder<'a> {
    pub(super) fn new(shared: &'a Client, bucket: &str) -> Self {
        let url = crate::url::UrlBuilder::new(bucket).format();

        Self {
            shared: Cow::Borrowed(shared),
            builder: shared.client.get(url),
        }
    }

    pub(super) fn new_buf(shared: &'a Client, url_buf: &mut String, bucket: &str) -> Self {
        crate::url::UrlBuilder::new(bucket).format_into(url_buf);

        Self {
            shared: Cow::Borrowed(shared),
            builder: shared.client.get(url_buf.as_str()),
        }
    }

    fn insert_query<P: Param>(self, name: &str, param: P) -> Self {
        Self {
            builder: param.append_param(name, self.builder),
            shared: self.shared,
        }
    }

    pub fn into_static(self) -> ListBuilder<'static> {
        ListBuilder {
            shared: Cow::Owned(self.shared.into_owned()),
            builder: self.builder,
        }
    }

    pub fn prefix<S: StringParam>(self, prefix: S) -> Self {
        self.insert_query("prefix", prefix)
    }

    pub fn delimiter<S: StringParam>(self, delimiter: S) -> Self {
        self.insert_query("delimiter", delimiter)
    }

    pub fn start_at<S: StringParam>(self, start_at: S) -> Self {
        self.insert_query("startOffset", start_at)
    }

    pub fn end_before<S: StringParam>(self, end_before: S) -> Self {
        self.insert_query("endOffset", end_before)
    }

    pub fn range<R, S>(mut self, range: R) -> Self
    where
        R: GcsBounds<S>,
        S: StringParam,
    {
        if let Some(start) = range.start_at() {
            self = self.start_at(start);
        }
        if let Some(end) = range.end_before() {
            self = self.end_before(end);
        }

        self
    }

    pub fn include_trailing_delimiter(self) -> Self {
        self.insert_query("includeTrailingDelimiter", true)
    }

    pub fn max_results(self, max: usize) -> Self {
        self.insert_query("maxResults", max.clamp(1, DEFAULT_MAX_PER_PAGE))
    }

    pub fn get(self) -> ListStream<'a> {
        let ListBuilder { shared, builder } = self;

        let (url, fut) = match builder.build() {
            Ok(mut request) => {
                let shared_clone = shared.clone();
                let url = request.url().clone();

                let fut = ReusableBoxFuture::new(async move {
                    let header = shared_clone.auth.get_header().await?;

                    request.headers_mut().insert(header::AUTHORIZATION, header);
                    crate::execute_and_validate_with_backoff(&shared_clone.client, request)
                        .await?
                        .json()
                        .await
                        .map_err(Error::from)
                });

                (Some(url), fut)
            }
            Err(error) => (
                None,
                ReusableBoxFuture::new(std::future::ready(Err(Error::from(error)))),
            ),
        };

        ListStream {
            url,
            shared,
            state: State::Requesting,
            fut,
        }
    }

    pub async fn get_all(self) -> Result<List, Error> {
        use futures::StreamExt;

        let stream = self.get();
        futures::pin_mut!(stream);

        let mut dst = match stream.next().await {
            Some(result) => result?,
            None => return Ok(List::default()),
        };

        while let Some(res) = stream.next().await {
            let List {
                mut prefixes,
                mut objects,
            } = res?;

            dst.prefixes.append(&mut prefixes);
            dst.objects.append(&mut objects);
        }

        Ok(dst)
    }
}

pin_project! {
    #[project = ListStreamProjection]
    pub struct ListStream<'a> {
        url: Option<reqwest::Url>,
        shared: Cow<'a, Client>,
        state: State,
        #[pin]
        fut: ReusableBoxFuture<'a, Result<RawList, Error>>,
    }
}

enum State {
    Done,
    Requesting,
    Error,
}

impl ListStreamProjection<'_, '_> {
    fn handle_fut_result(
        &mut self,
        result: Result<RawList, Error>,
        cx: &mut Context<'_>,
    ) -> Result<List, Error> {
        match result {
            // if there is no next-page-token:
            Ok(RawList {
                next_page_token: None,
                prefixes,
                objects,
            }) => {
                *self.state = State::Done;
                Ok(List { prefixes, objects })
            }
            // if there is a next page token, aka, we need to request the next page.
            Ok(RawList {
                next_page_token: Some(token),
                mut prefixes,
                mut objects,
            }) => {
                // this block will only execute if we make the request for the next page, and it
                // completes instantly. Very very unlikly (and if it does happen, it's almost
                // definitely going to be an error, since the OK case needs to make a network
                // request)
                if let Some(result) = self.kick_off_next_request(token, cx) {
                    match result {
                        // if miraculously we get something, just append it to the set of results we
                        // already have.
                        Ok(List {
                            prefixes: mut new_prefixes,
                            objects: mut new_objects,
                        }) => {
                            prefixes.append(&mut new_prefixes);
                            objects.append(&mut new_objects);
                        }
                        Err(err) => {
                            *self.state = State::Error;
                            return Err(err);
                        }
                    }
                }
                Ok(List { prefixes, objects })
            }
            Err(error) => {
                *self.state = State::Error;
                Err(error)
            }
        }
    }

    fn kick_off_next_request(
        &mut self,
        token: String,
        cx: &mut Context<'_>,
    ) -> Option<Result<List, Error>> {
        let url = self
            .url
            .as_ref()
            .expect("this will be Some unless the initial request fails");

        self.fut
            .as_mut()
            .get_mut()
            .set(request_next_page(self.shared.clone(), url, token));

        // since we need to poll once, we need to handle the case where the future completes
        // instantly. I cant imagine that happening in reality (since its a networking
        // call), but nonetheless, do things recursively.
        match self.fut.as_mut().poll(cx) {
            Poll::Pending => None,
            Poll::Ready(result) => Some(self.handle_fut_result(result, cx)),
        }
    }
}

fn request_next_page<'a>(
    shared: Cow<'a, Client>,
    url: &reqwest::Url,
    token: String,
) -> impl std::future::Future<Output = Result<RawList, Error>> + 'a {
    let builder = shared.client.get(url.clone());
    async move {
        let auth_header = shared.auth.get_header().await?;

        let request = builder
            .header(header::AUTHORIZATION, auth_header)
            .query(&[("pageToken", token)])
            .build()?;

        crate::execute_and_validate_with_backoff(&shared.client, request)
            .await?
            .json()
            .await
            .map_err(Error::from)
    }
}

impl<'a> Stream for ListStream<'a> {
    type Item = Result<List, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match *this.state {
            // hitting an error will counts as 'tripping' the stream, so we'll never
            // return more after an error.
            State::Done | State::Error => Poll::Ready(None),
            State::Requesting => {
                let result = ready!(this.fut.as_mut().poll(cx));
                Poll::Ready(Some(this.handle_fut_result(result, cx)))
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct List {
    pub prefixes: Vec<String>,
    pub objects: Vec<Object>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawList {
    next_page_token: Option<String>,
    #[serde(default)]
    prefixes: Vec<String>,
    #[serde(default, rename = "items")]
    objects: Vec<Object>,
}

/// Since GCS only supports inclusive start bounds, and exclusive end bounds (or no bound at all),
/// this trait  is only implemented on std [`RangeBounds`] that fit that description
/// ([`Range`], [`RangeFull`], [`RangeTo`] and [`RangeFrom`])
///
/// [`RangeBounds`]: std::ops::RangeBounds
/// [`Range`]: std::ops::Range
/// [`RangeFull`]: std::ops::RangeFull
/// [`RangeTo`]: std::ops::RangeTo
/// [`RangeFrom`]: std::ops::RangeFrom
pub trait GcsBounds<S: StringParam + ?Sized> {
    fn start_at(&self) -> Option<&S>;

    fn end_before(&self) -> Option<&S>;
}

impl<T, S> GcsBounds<S> for &T
where
    T: GcsBounds<S> + ?Sized,
    S: StringParam + ?Sized,
{
    fn start_at(&self) -> Option<&S> {
        T::start_at(self)
    }

    fn end_before(&self) -> Option<&S> {
        T::end_before(self)
    }
}

impl<S: StringParam + ?Sized> GcsBounds<S> for std::ops::RangeFull {
    fn start_at(&self) -> Option<&S> {
        None
    }

    fn end_before(&self) -> Option<&S> {
        None
    }
}

impl<S: StringParam> GcsBounds<S> for std::ops::Range<S> {
    fn start_at(&self) -> Option<&S> {
        Some(&self.start)
    }

    fn end_before(&self) -> Option<&S> {
        Some(&self.end)
    }
}

impl<S: StringParam> GcsBounds<S> for std::ops::RangeTo<S> {
    fn start_at(&self) -> Option<&S> {
        None
    }

    fn end_before(&self) -> Option<&S> {
        Some(&self.end)
    }
}

impl<S: StringParam> GcsBounds<S> for std::ops::RangeFrom<S> {
    fn start_at(&self) -> Option<&S> {
        Some(&self.start)
    }

    fn end_before(&self) -> Option<&S> {
        None
    }
}
