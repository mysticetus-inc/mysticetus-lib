use std::future::Future;
use std::ops::RangeBounds;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use protos::storage::{ListObjectsRequest, Object};
use tokio_util::sync::ReusableBoxFuture;

use crate::bucket::BucketClient;
use crate::util::OwnedOrMut;

pub const MAX_PAGE_SIZE: u32 = 1000;

pub struct ListBuilder<'a> {
    client: OwnedOrMut<'a, BucketClient>,
    page_size: u32,
    prefix: Option<String>,
    delimiter: Option<String>,
    include_trailing_delimiter: bool,
    prefetch_next_chunk: bool,
    glob: Option<String>,
    lexicographic_start: Option<String>,
    lexicographic_end: Option<String>,
}

impl<'a> ListBuilder<'a> {
    pub(crate) fn new(client: impl Into<OwnedOrMut<'a, BucketClient>>) -> Self {
        Self {
            client: client.into(),
            page_size: MAX_PAGE_SIZE,
            delimiter: None,
            include_trailing_delimiter: false,
            glob: None,
            prefix: None,
            prefetch_next_chunk: true,
            lexicographic_end: None,
            lexicographic_start: None,
        }
    }

    /// Matches items based on a glob pattern. The specific syntax is detailed here:
    /// <https://cloud.google.com/storage/docs/json_api/v1/objects/list#list-objects-and-prefixes-using-glob>
    pub fn glob(mut self, glob: impl Into<String>) -> Self {
        self.glob = Some(glob.into());
        self
    }

    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    pub fn include_trailing_delimiter(mut self) -> Self {
        self.include_trailing_delimiter = true;
        self
    }

    /// Shorthand for <code>list_builder.include_trailing_delimiters().delimiter("/")</code>
    pub fn folders(mut self) -> Self {
        self.delimiter = Some("/".to_owned());
        self.include_trailing_delimiter = true;
        self
    }

    pub fn page_size(mut self, page_size: u32) -> Self {
        assert_ne!(page_size, 0, "page size can't be 0");
        self.page_size = page_size.clamp(1, MAX_PAGE_SIZE);
        self
    }

    pub fn range<S: AsRef<str>>(mut self, range: impl RangeBounds<S>) -> Self {
        macro_rules! append_char {
            ($s:expr, $ch:expr) => {{
                let mut s: String = $s.as_ref().to_owned();
                s.push($ch);
                s
            }};
        }

        self.lexicographic_start = match range.start_bound() {
            std::ops::Bound::Unbounded => None,
            std::ops::Bound::Excluded(excl) => Some(append_char!(excl, char::MIN)),
            std::ops::Bound::Included(incl) => Some(incl.as_ref().to_owned()),
        };

        self.lexicographic_end = match range.end_bound() {
            std::ops::Bound::Unbounded => None,
            std::ops::Bound::Excluded(excl) => Some(excl.as_ref().to_owned()),
            std::ops::Bound::Included(incl) => Some(append_char!(incl, char::MAX)),
        };

        self
    }

    pub fn prefix<P>(mut self, prefix: P) -> Self
    where
        P: Into<String>,
    {
        self.prefix = Some(prefix.into());
        self
    }

    fn into_parts(self) -> (ListObjectsRequest, OwnedOrMut<'a, BucketClient>) {
        let request = ListObjectsRequest {
            parent: self.client.qualified_bucket().to_owned(),
            page_size: self.page_size as i32,
            soft_deleted: false,
            page_token: String::new(),
            include_trailing_delimiter: self.include_trailing_delimiter,
            prefix: self.prefix.unwrap_or_default(),
            versions: false,
            filter: String::new(),
            read_mask: None,
            lexicographic_end: self.lexicographic_end.unwrap_or_default(),
            lexicographic_start: self.lexicographic_start.unwrap_or_default(),
            match_glob: self.glob.unwrap_or_default(),
            include_folders_as_prefixes: self.include_trailing_delimiter
                && self.delimiter.as_ref().is_some_and(|delim| delim == "/"),
            delimiter: self.delimiter.unwrap_or_default(),
        };

        (request, self.client)
    }

    /// Only performs the request for the first page of results, ignoring
    /// any further pagination that might be possible.
    pub async fn first_page(self) -> crate::Result<ListObjects> {
        let (request, client) = self.into_parts();
        let (_, objects, _) = make_request(client, request).await?;
        Ok(objects)
    }

    #[must_use = "ListStream needs to be polled to start the initial request"]
    pub fn get(self) -> ListStream<'a> {
        let prefetch_next_chunk = self.prefetch_next_chunk;
        let (request, client) = self.into_parts();

        ListStream {
            next_request_parts: None,
            prefetch_next_chunk,
            state: FutureState::Requesting,
            future: ReusableBoxFuture::new(make_request(client, request)),
        }
    }

    pub async fn collect(self) -> crate::Result<ListObjects> {
        self.get().collect().await
    }
}
// make all requests via this function, that way the ResuableBoxFuture
// should be able to be reused across every request (ideally)
async fn make_request(
    mut client: OwnedOrMut<'_, BucketClient>,
    mut request: ListObjectsRequest,
) -> crate::Result<(
    OwnedOrMut<'_, BucketClient>,
    ListObjects,
    Option<ListObjectsRequest>,
)> {
    // we need to copy the entire request, paginating with only the page token doesn't
    // respect the original request
    let results = client
        .client_mut()
        .list_objects(request.clone())
        .await?
        .into_inner();

    let list = ListObjects {
        objects: results.objects,
        prefixes: results.prefixes,
    };

    let next_request = if results.next_page_token.is_empty() {
        None
    } else {
        request.page_token = results.next_page_token;
        Some(request)
    };

    Ok((client, list, next_request))
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ListObjects {
    pub objects: Vec<Object>,
    pub prefixes: Vec<String>,
}

impl ListObjects {
    pub fn extend(&mut self, mut other: ListObjects) {
        self.objects.append(&mut other.objects);
        self.prefixes.append(&mut other.prefixes);
    }
}

pin_project_lite::pin_project! {
    #[project = ListStreamProjection]
    pub struct ListStream<'a> {
        next_request_parts: Option<(OwnedOrMut<'a, BucketClient>, ListObjectsRequest)>,
        prefetch_next_chunk: bool,
        state: FutureState,
        #[pin]
        future: ReusableBoxFuture<'a, crate::Result<(OwnedOrMut<'a, BucketClient>, ListObjects, Option<ListObjectsRequest>)>>
    }
}

impl ListStream<'_> {
    pub async fn collect(mut self) -> crate::Result<ListObjects> {
        // if we're gonna be collecting all the results consecutively,
        // might as well override this
        self.prefetch_next_chunk = true;

        let mut dst = match self.next_page().await? {
            Some(dst) => dst,
            None => return Ok(ListObjects::default()),
        };

        while let Some(next_page) = self.next_page().await? {
            dst.extend(next_page);
        }

        Ok(dst)
    }

    /// Convinence method. (basically [`futures::StreamExt::next`] but without
    /// needing to import it, plus the fn name is a bit more clear)
    pub async fn next_page(&mut self) -> crate::Result<Option<ListObjects>> {
        let mut pinned = Pin::new(self);

        std::future::poll_fn(|cx| pinned.as_mut().poll_next(cx))
            .await
            .transpose()
    }
}

enum FutureState {
    Requesting,
    Done,
}

impl<'a> ListStreamProjection<'_, 'a> {
    fn start_next_request(
        &mut self,
        client: OwnedOrMut<'a, BucketClient>,
        next_request: ListObjectsRequest,
    ) {
        self.future
            .as_mut()
            .get_mut()
            .set(make_request(client, next_request));

        *self.state = FutureState::Requesting;
    }
}

impl<'a> Stream for ListStream<'a> {
    type Item = crate::Result<ListObjects>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let mut list_results: Option<ListObjects> = None;

        loop {
            match this.state {
                FutureState::Done => match this.next_request_parts.take() {
                    // if the future isnt pending and we have no next page token, we're done
                    None => break,
                    Some((client, page_token)) => this.start_next_request(client, page_token),
                },
                FutureState::Requesting => {
                    let Poll::Ready(result) = this.future.as_mut().poll(cx) else {
                        break;
                    };

                    *this.state = FutureState::Done;

                    let (client, objects, next_request) = result?;

                    match list_results {
                        Some(ref mut existing) => existing.extend(objects),
                        None => list_results = Some(objects),
                    }
                    match (*this.prefetch_next_chunk, next_request) {
                        (true, Some(request)) => this.start_next_request(client, request),
                        (false, Some(request)) => {
                            *this.next_request_parts = Some((client, request));
                            break;
                        }
                        (_, None) => break,
                    }
                }
            }
        }

        match list_results.take() {
            Some(results) => Poll::Ready(Some(Ok(results))),
            None if matches!(this.state, FutureState::Done) => Poll::Ready(None),
            None => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let minimum = match (&self.next_request_parts, &self.state) {
            (Some(_), FutureState::Requesting) => 2,
            (None, FutureState::Requesting) | (Some(_), FutureState::Done) => 1,
            // special case when the stream is exhausted
            (None, FutureState::Done) => return (0, Some(0)),
        };

        (minimum, None)
    }
}
