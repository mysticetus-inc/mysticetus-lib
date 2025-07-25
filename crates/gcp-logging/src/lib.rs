mod http_request;
mod json;
mod middleware;
pub mod options;
mod payload;
mod records;
mod subscriber;
mod types;
mod utils;

pub use options::{DefaultLogOptions, LogOptions};
pub use subscriber::builder::LoggingBuilder;
// re-export `tracing` and `tracing-subscriber`
pub use tracing;
pub use tracing_subscriber;

#[cfg(test)]
mod test_utils;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    #[default]
    Dev,
    Test,
    Production,
}

/// Casts an [`std::error::Error`] type into a log-able [`tracing::Value`],
/// via the [`&(dyn std::error::Error + 'static)`] impl
pub fn err<E>(error: &E) -> impl tracing::Value + '_
where
    E: std::error::Error + 'static,
{
    error as &(dyn std::error::Error + 'static)
}

#[macro_export]
macro_rules! alert {
    ($error:expr, $($t:tt)+) => {
        $crate::tracing::error!(error = $crate::err($error), alert = true, $($t)+)
    };
    ($error:expr $(,)?) => {
        $crate::tracing::error!(message = "fatal error", error = $crate::err($error), alert = true)
    };
}

#[cfg(test)]
mod tests {
    use test_utils::{EchoService, EmptyBody, MakeTestWriter};
    use tower::{Layer, Service};
    use tracing_subscriber::util::SubscriberInitExt;

    use super::*;

    const TEST_HEADER: http::HeaderValue = http::HeaderValue::from_static("gcp-logging-test");
    const TEST_URI: &str = "https://a-real-uri.com";

    use crate::http_request::TRACE_CTX_HEADER;
    const TEST_TRACE_VALUE: http::HeaderValue = http::HeaderValue::from_static("test-trace");

    fn make_request_and_service() -> (
        http::Request<EmptyBody>,
        EchoService<for<'a> fn(&'a http::Request<EmptyBody>)>,
    ) {
        let req = http::Request::builder()
            .header(http::header::ORIGIN, TEST_HEADER)
            .header(TRACE_CTX_HEADER, TEST_TRACE_VALUE)
            .uri(TEST_URI)
            .body(EmptyBody)
            .expect("should be valid");

        fn echo_inner(req: &http::Request<EmptyBody>) {
            assert_eq!(req.uri(), TEST_URI);
            assert_eq!(req.headers().get(http::header::ORIGIN), Some(&TEST_HEADER));
            tracing::info!(
                message = "got request",
                uri = %req.uri(),
                headers = ?req.headers(),
                label.echo = "echo",
            );
        }

        (req, EchoService(echo_inner))
    }

    macro_rules! run_in_new_span {
        ($span_name:literal[$($field:ident $(. $subfield:ident)? = $value:expr),* $(,)?] $b:block) => {{
            let span = tracing::info_span!($span_name, $($field$(.$subfield)? = $value,)*);

            use tracing::Instrument;

           let ret = (async move { $b }).instrument(span).await;
           println!("dropped {}", $span_name);
           ret
        }};
    }

    #[tokio::test]
    async fn test_json_format() {
        let (rx, make_writer) = MakeTestWriter::<false>::new();

        let subscriber = LoggingBuilder::new_from_stage(Stage::Test)
            .project_id("mysticetus-oncloud")
            .with_writer(make_writer)
            .build();

        let handle = subscriber.handle();

        let _default_guard = subscriber.set_default();

        let (req, svc) = make_request_and_service();

        // artificially nest in a bunch of spans, that way we can test
        // span/scope iteration, etc
        run_in_new_span! {
            "first_span"[first = true] {
                run_in_new_span! {
                    "second_span"[second = true, label.test = true] {
                        run_in_new_span! {
                            "innermost_span"[last = true, field = "test"] {
                                let mut svc = handle.layer(svc);
                                _ = svc.call(req).await.unwrap();
                            }
                        }
                    }
                }
            }
        }

        let events = rx.try_iter().collect::<Vec<Vec<_>>>();

        for (i, event) in events.iter().enumerate() {
            std::fs::write(format!("new_event_json_bytes_{i}.json"), event).unwrap();
        }
    }
}
