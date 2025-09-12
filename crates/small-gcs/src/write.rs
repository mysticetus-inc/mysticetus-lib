use std::borrow::Cow;
use std::path::Path;

use bytes::Bytes;
use futures::TryStream;
use mime_guess::Mime;
use net_utils::backoff::Backoff;
use reqwest::RequestBuilder;
use reqwest::header::{self, HeaderValue};
use tokio_util::io::ReaderStream;

use crate::client::Client;
use crate::url::UrlBuilder;
use crate::{Error, Object};

/// A builder type for constructing an [`Object`] upload. Built by the
/// [`StorageClient::write`] method.
pub struct WriteBuilder<'a, L> {
    shared: &'a Client,
    builder: RequestBuilder,
    name: &'a str,
    mime: Option<MimeOrString>,
    content_len: L,
}

impl<'a> WriteBuilder<'a, ()> {
    #[inline]
    pub(super) fn new(shared: &'a Client, bucket: &str, name: &'a str) -> Self {
        let url = UrlBuilder::new(bucket).upload().format();

        Self {
            builder: shared.client.post(url),
            content_len: (),
            shared,
            name,
            mime: None,
        }
    }

    #[inline]
    pub(super) fn new_buf(
        shared: &'a Client,
        url_buf: &mut String,
        bucket: &str,
        name: &'a str,
    ) -> Self {
        // object name/path isnt part of the URL path for uploads, instead its a query
        // parameter for some reason, so it gets added in the actual upload method
        UrlBuilder::new(bucket).upload().format_into(url_buf);

        Self {
            builder: shared.client.post(url_buf.as_str()),
            content_len: (),
            shared,
            name,
            mime: None,
        }
    }

    /// Uploads from a file on disc.
    pub async fn file<P: AsRef<Path>>(self, file_path: P) -> Result<Object, Error> {
        self.file_inner(file_path.as_ref()).await
    }

    /// Non-generic inner function for 'file'
    async fn file_inner(self, path: &Path) -> Result<Object, Error> {
        let file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;

        let capacity = {
            // if we can get the block size, use that, otherwise default to 4Kib.
            #[cfg(target_family = "unix")]
            {
                use std::os::unix::fs::MetadataExt;
                metadata.blksize()
            }
            #[cfg(not(target_family = "unix"))]
            {
                4096
            }
        };
        let reader = ReaderStream::with_capacity(file, capacity as usize);

        self.content_len(metadata.len())
            .upload_streamed(reader)
            .await
    }

    pub fn content_len(self, content_len: u64) -> WriteBuilder<'a, u64> {
        WriteBuilder {
            shared: self.shared,
            builder: self.builder,
            name: self.name,
            mime: self.mime,
            content_len,
        }
    }
}

impl<'a, L> WriteBuilder<'a, L> {
    /// Adds a request precondition, which will fail if the existing generation doesn't match the
    /// given 'generation'. Opposite of [`WriteBuilder::if_generation_not_match`].
    ///
    /// For the special-cased '0' generation, use [`WriteBuilder::if_does_not_exist`] since it's a
    /// bit clearer.
    pub fn if_generation_match(self, generation: usize) -> Self {
        Self {
            builder: self.builder.query(&[("ifGenerationMatch", generation)]),
            ..self
        }
    }

    /// Adds a request precondition that will fail if there's already an object at the path that
    /// this object tries to upload to. Opposite of [`WriteBuilder::if_does_not_exist`]
    pub fn if_already_exists(self) -> Self {
        self.if_generation_not_match(0)
    }

    /// Adds a request precondition that will fail if there __isn't__ already an object at
    /// the path that this object tries to upload to. Opposite of
    /// [`WriteBuilder::if_already_exists`]
    pub fn if_does_not_exist(self) -> Self {
        self.if_generation_match(0)
    }

    /// Adds a request precondition, which will fail if the existing generation matches the
    /// given 'generation'. Opposite of [`WriteBuilder::if_generation_match`].
    ///
    /// For the special-cased '0' generation, use [`WriteBuilder::if_already_exists`] since it's a
    /// bit clearer.
    pub fn if_generation_not_match(self, generation: usize) -> Self {
        Self {
            builder: self.builder.query(&[("ifGenerationNotMatch", generation)]),
            ..self
        }
    }

    pub fn mime_type<M>(mut self, mime: M) -> Self
    where
        M: Into<MimeOrString>,
    {
        self.mime = Some(mime.into());
        self
    }

    /// Uploads an object from an in-memory buffer.
    pub async fn upload<C>(self, content: C) -> Result<Object, Error>
    where
        Bytes: From<C>,
    {
        let bytes = Bytes::from(content);
        let len = bytes.len();

        upload_body_inner(
            self.shared,
            self.builder,
            self.mime,
            self.name,
            reqwest::Body::from(bytes),
            len as u64,
        )
        .await
    }
}

impl<'a> WriteBuilder<'a, u64> {
    /// Uploads an object from a stream of bytes.
    pub async fn upload_streamed<S>(self, stream: S) -> Result<Object, Error>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        upload_body_inner(
            self.shared,
            self.builder,
            self.mime,
            self.name,
            reqwest::Body::wrap_stream(stream),
            self.content_len,
        )
        .await
    }
}

/// Non-generic inner upload method that all upload methods call under the hood.
async fn upload_body_inner(
    shared: &Client,
    builder: RequestBuilder,
    mime: Option<MimeOrString>,
    name: &str,
    body: reqwest::Body,
    len: u64,
) -> Result<Object, Error> {
    // build/parse the content type header value, from either
    // a user supplied mime/str, or by guessing from the path.
    // defaults to 'application/octet-stream' if the path/extension
    // doesn't have an obvious mime type.
    let content_type = match mime {
        Some(MimeOrString::String(s)) => HeaderValue::from_str(&s)?,
        Some(MimeOrString::Mime(mime)) => HeaderValue::from_str(mime.as_ref())?,
        None => {
            let mime = mime_guess::from_path(name).first_or_octet_stream();
            HeaderValue::from_str(mime.as_ref())?
        }
    };

    let auth = shared.auth.get_header().into_header().await?;
    let request = builder
        .query(&[("name", name)]) // reqwest handles encoding query params, so we dont need to do it here
        .header(header::AUTHORIZATION, auth.header)
        .header(header::CONTENT_LENGTH, len)
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .build()?;

    let resp = crate::try_execute_with_backoff(&shared, request, Backoff::default).await?;

    let ok_resp = crate::validate_response(resp).await?;

    ok_resp.json().await.map_err(Error::from)
}

/// Helper type for using either a parsed [`Mime`], or
/// just defering to a basic [`&'static str`] or [`String`] mime type.
pub enum MimeOrString {
    String(Cow<'static, str>),
    Mime(Mime),
}

impl From<Cow<'static, str>> for MimeOrString {
    fn from(value: Cow<'static, str>) -> Self {
        Self::String(value)
    }
}

impl From<&'static str> for MimeOrString {
    fn from(value: &'static str) -> Self {
        Cow::Borrowed(value).into()
    }
}

impl From<String> for MimeOrString {
    fn from(value: String) -> Self {
        MimeOrString::String(Cow::Owned(value))
    }
}

impl From<Mime> for MimeOrString {
    fn from(value: Mime) -> Self {
        Self::Mime(value)
    }
}

impl From<mime_guess::MimeGuess> for MimeOrString {
    fn from(value: mime_guess::MimeGuess) -> Self {
        value.first_or_octet_stream().into()
    }
}
