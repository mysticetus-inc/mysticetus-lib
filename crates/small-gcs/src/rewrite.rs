use std::borrow::Cow;

use reqwest::header;
use serde::Deserialize;
use timestamp::Duration;

use crate::{Client, Error, Object};

const MIN_REWRITE_TIMEOUT: Duration = Duration::from_seconds(30);

pub struct RewriteToBuilder<'a> {
    shared: &'a Client,
    src_bucket: &'a str,
    src_name: &'a str,
    buf: Option<&'a mut String>,
}

impl<'a> RewriteToBuilder<'a> {
    pub(crate) const fn new(
        shared: &'a Client,
        src_bucket: &'a str,
        src_name: &'a str,
        buf: Option<&'a mut String>,
    ) -> Self {
        Self {
            shared,
            src_bucket,
            src_name,
            buf,
        }
    }

    pub fn to<B, P>(self, dst_bucket: B, dst_name: P) -> RewriteBuilder<'a>
    where
        Cow<'a, str>: From<B> + From<P>,
    {
        RewriteBuilder {
            shared: Cow::Borrowed(self.shared),
            src_name: Cow::Borrowed(self.src_name),
            src_bucket: Cow::Borrowed(self.src_bucket),
            dst_bucket: Cow::from(dst_bucket),
            dst_name: Cow::from(dst_name),
            buf: self.buf.map(StringBuf::Ref),
            poll_interval: MIN_REWRITE_TIMEOUT,
        }
    }
}

pub struct RewriteBuilder<'a> {
    shared: Cow<'a, Client>,
    src_name: Cow<'a, str>,
    src_bucket: Cow<'a, str>,
    dst_bucket: Cow<'a, str>,
    dst_name: Cow<'a, str>,
    buf: Option<StringBuf<'a>>,
    poll_interval: Duration,
}

enum StringBuf<'a> {
    Owned(String),
    Ref(&'a mut String),
}

impl StringBuf<'_> {
    fn as_str(&self) -> &str {
        match self {
            Self::Owned(owned) => owned,
            Self::Ref(refer) => refer,
        }
    }
    fn as_mut(&mut self) -> &mut String {
        match self {
            Self::Owned(owned) => owned,
            Self::Ref(refer) => refer,
        }
    }
    fn into_owned<'a>(self) -> StringBuf<'a> {
        match self {
            Self::Owned(owned) => StringBuf::Owned(owned),
            Self::Ref(refer) => StringBuf::Owned(refer.clone()),
        }
    }
}

#[derive(Debug)]
pub enum Rewrite<'a> {
    Done(Object),
    Longrunning(LongrunningRewrite<'a>),
}

impl<'a> Rewrite<'a> {
    pub async fn wait(&mut self) -> Result<&Object, Error> {
        match self {
            Self::Done(done) => Ok(done),
            Self::Longrunning(lr) => {
                let obj = lr.wait().await?;
                *self = Self::Done(obj);

                match self {
                    Self::Done(d) => Ok(d),
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct LongrunningRewrite<'a> {
    shared: Cow<'a, Client>,
    request: reqwest::Request,
    interval: tokio::time::Interval,
    last_resp: RewriteResponse,
}

impl LongrunningRewrite<'_> {
    pub async fn wait(&mut self) -> Result<Object, Error> {
        loop {
            let progress = self.poll_status().await?;

            if progress.done {
                return self
                    .last_resp
                    .resource
                    .take()
                    .ok_or_else(Error::missing_resource);
            }
        }
    }

    pub async fn poll_status(&mut self) -> Result<Progress, Error> {
        self.interval.tick().await;
        self.force_poll_status().await
    }

    pub fn last_progress(&self) -> Progress {
        Progress {
            done: self.last_resp.done,
            total_bytes_written: self.last_resp.total_bytes_rewritten,
            object_size: self.last_resp.object_size,
        }
    }

    pub async fn force_poll_status(&mut self) -> Result<Progress, Error> {
        let Some(token) = self.last_resp.rewrite_token.as_deref() else {
            return Ok(Progress {
                done: true,
                total_bytes_written: 0,
                object_size: 0,
            });
        };

        let request = self
            .request
            .try_clone()
            .expect("get request has no body, therefore is clonable");

        self.last_resp = call_rewrite(&self.shared, request, Some(token)).await?;
        println!("{:#?}", self.last_resp);
        Ok(self.last_progress())
    }
}

async fn call_rewrite(
    client: &Client,
    mut request: reqwest::Request,
    token: Option<&str>,
) -> Result<RewriteResponse, Error> {
    if let Some(token) = token {
        request
            .url_mut()
            .query_pairs_mut()
            .append_pair("rewriteToken", token)
            .finish();
    }

    crate::execute_and_validate_with_backoff(&client.client, request)
        .await?
        .json()
        .await
        .map_err(Error::Reqwest)
}

impl<'a> RewriteBuilder<'a> {
    pub fn poll_interval(mut self, dur: Duration) -> Self {
        self.poll_interval = dur.max(MIN_REWRITE_TIMEOUT);
        self
    }

    pub fn into_owned(self) -> RewriteBuilder<'static> {
        RewriteBuilder {
            shared: Cow::Owned(self.shared.into_owned()),
            src_name: Cow::Owned(self.src_name.into_owned()),
            src_bucket: Cow::Owned(self.src_bucket.into_owned()),
            dst_bucket: Cow::Owned(self.dst_bucket.into_owned()),
            dst_name: Cow::Owned(self.dst_name.into_owned()),
            buf: self.buf.map(|buf| buf.into_owned()),
            poll_interval: MIN_REWRITE_TIMEOUT,
        }
    }

    pub async fn send(self) -> Result<Rewrite<'a>, Error> {
        let mut url_builder = crate::url::UrlBuilder::new(&self.src_bucket)
            .name(&self.src_name)
            .rewrite(&self.dst_bucket, &self.dst_name);

        let url = match self.buf {
            Some(mut buf) => {
                url_builder.format_into(buf.as_mut());
                buf
            }
            None => StringBuf::Owned(url_builder.format()),
        };

        let auth_header = self.shared.auth.get_header().await?;

        let request = self
            .shared
            .client
            .post(url.as_str())
            .query(&[("maxBytesRewrittenPerCall", "1048576")])
            .header(header::CONTENT_LENGTH, "0")
            .header(header::AUTHORIZATION, auth_header)
            .build()?;

        let cloned_request = request.try_clone().expect("no body, should be clonable");
        let last_resp = call_rewrite(&self.shared, request, None).await?;

        if last_resp.done {
            let resource = last_resp.resource.ok_or_else(Error::missing_resource)?;

            Ok(Rewrite::Done(resource))
        } else {
            let mut interval = tokio::time::interval(self.poll_interval.into());
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            // since the first tick completes right away, we don't want end users to be able
            // to poll instantly or google might get mad. shouldn't block at all.
            interval.tick().await;

            Ok(Rewrite::Longrunning(LongrunningRewrite {
                shared: self.shared,
                request: cloned_request,
                interval,
                last_resp,
            }))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Progress {
    pub done: bool,
    pub total_bytes_written: u64,
    pub object_size: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RewriteResponse {
    #[serde(deserialize_with = "serde_helpers::from_str_visitor::FromStrVisitor::deserialize")]
    total_bytes_rewritten: u64,
    #[serde(deserialize_with = "serde_helpers::from_str_visitor::FromStrVisitor::deserialize")]
    object_size: u64,
    done: bool,
    rewrite_token: Option<String>,
    resource: Option<Object>,
}
