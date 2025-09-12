#![feature(const_trait_impl, linked_list_cursors)]

mod client;
pub mod list;
// mod multipart;
mod object;
mod query_param;
mod read;
mod rewrite;
mod url;
mod write;

pub mod error;
pub use client::{BucketClient, Client};
pub use error::Error;
pub(crate) use error::validate_response;
pub use list::ListBuilder;
use net_utils::backoff::Backoff;
pub use object::{NewObject, Object};
pub use read::ReadBuilder;
pub use rewrite::RewriteBuilder;
pub use write::WriteBuilder;

pub mod params {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Alt {
        Json,
        Media,
    }

    impl Alt {
        fn as_str(self) -> &'static str {
            match self {
                Self::Json => "json",
                Self::Media => "media",
            }
        }
    }

    impl serde::Serialize for Alt {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            ("alt", self.as_str()).serialize(serializer)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UploadType {
        Media,
        Multipart,
        Resumable,
    }

    impl serde::Serialize for UploadType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::Media => ("uploadType", "media").serialize(serializer),
                Self::Multipart => ("uploadType", "multipart").serialize(serializer),
                Self::Resumable => ("uploadType", "resumable").serialize(serializer),
            }
        }
    }
}

async fn try_execute_with_backoff<F>(
    client: &Client,
    request: reqwest::Request,
    backoff_builder: F,
) -> Result<reqwest::Response, Error>
where
    F: FnOnce() -> Backoff,
{
    // helper that tries to clone a request, then make said request with the original.
    async fn execute_once(
        client: &Client,
        mut request: reqwest::Request,
    ) -> Result<(Option<reqwest::Request>, reqwest::Response), Error> {
        let cloned = request.try_clone();

        if let reqwest::header::Entry::Vacant(vacant) =
            request.headers_mut().entry(reqwest::header::AUTHORIZATION)
        {
            let header = client.auth.get_header().await?;
            vacant.insert(header);
        }

        let resp = client.client.execute(request).await?;
        Ok((cloned, resp))
    }

    /// bails with the last response if we either couldnt clone the request or the response didn't
    /// meet one of the 2 criteria for being retried:
    ///
    /// - A 429 (too many requests) status
    /// - A 401 | 403 (auth error)
    /// - A 5XX server error, on google's end
    macro_rules! bail_if_no_retry {
        ($maybe_request:expr, $resp:expr) => {{
            match (&mut $maybe_request, $resp.status().as_u16()) {
                (Some(_), 429 | 500..=599) => (),
                (Some(request), 401 | 403) => {
                    client.auth.revoke_token(true);
                    request.headers_mut().remove(reqwest::header::AUTHORIZATION);
                }
                _ => return Ok($resp),
            }
        }};
    }

    let (mut maybe_request, mut last_response) = execute_once(&client, request).await?;

    bail_if_no_retry!(maybe_request, last_response);

    // if we're here, we need to start backing off, so build the backoff type.
    let mut backoff = backoff_builder();

    // start looping for the successive retries
    while let Some(request) = maybe_request.take() {
        // if we have no more backoffs remaining, return the last response.
        // Otherwise wait one backoff cycle, and try again.
        match backoff.backoff_once() {
            Some(once) => once.await,
            None => return Ok(last_response),
        }

        (maybe_request, last_response) = execute_once(&client, request).await?;
        bail_if_no_retry!(maybe_request, last_response);
    }

    // if we run out of retries, we bail from within the while loop. similarly,
    // if the request can't be cloned, the 'bail_if_no_retry!' will return,
    // so 'maybe_request' should always be Some when the loop starts over.
    unreachable!()
}

/// Combines [`try_execute_with_backoff`] and [`validate_response`] into 1 function call.
async fn execute_and_validate_with_backoff(
    client: &Client,
    request: reqwest::Request,
) -> Result<reqwest::Response, Error> {
    use futures::TryFutureExt;

    try_execute_with_backoff(client, request, Backoff::default)
        .and_then(validate_response)
        .await
}

#[tokio::test]
async fn test_client() -> Result<(), Error> {
    let path = "VNE.0522.Fugro.RPS.GOExplorer.GP.Jul.2022-443155/SignOffs/GOExplorerVis/\
                2022-10-21/GOExplorerVis-2022-10-21-2358-Final-Edited-EPE-epe-KD-epe-[KD].\
                Mysticetus";
    let mut client = BucketClient::new(
        "mysticetus-oncloud",
        "mysticetus-replicated-data",
        gcp_auth_channel::Scope::GcsReadWrite,
    )
    .await?;

    let rewrite = client
        .rewrite(path)
        .to("mysticetus-gcr-logs", "rewritten.mysticetus")
        .send()
        .await?;

    println!("{rewrite:#?}");

    let mut long = match rewrite {
        rewrite::Rewrite::Done(obj) => {
            println!("1 pull: {obj:#?}");
            return Ok(());
        }
        rewrite::Rewrite::Longrunning(longrunning) => longrunning,
    };

    println!("{long:#?}");

    loop {
        let progress = long.poll_status().await?;

        println!("{progress:#?}");
        if progress.done {
            break;
        }
    }

    Ok(())
}
