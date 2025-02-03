use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::Stream;
use http::HeaderMap;
use http_body::Body;
use hyper::body::Incoming;

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct BodyStream {
        #[pin]
        pub incoming: Option<Incoming>,
        pub trailers: Option<HeaderMap>,
        _priv: (),
    }
}

impl BodyStream {
    #[inline]
    pub const fn new(incoming: Incoming) -> Self {
        Self {
            incoming: Some(incoming),
            trailers: None,
            _priv: (),
        }
    }
}

impl Stream for BodyStream {
    type Item = Result<<Incoming as Body>::Data, <Incoming as Body>::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        while let Some(incoming) = this.incoming.as_mut().as_pin_mut() {
            let Some(frame) = ready!(incoming.poll_frame(cx)).transpose()? else {
                this.incoming.set(None);
                break;
            };

            let frame = match frame.into_data() {
                Ok(data) => return Poll::Ready(Some(Ok(data))),
                Err(frame) => frame,
            };

            // this should always return Ok, since we already checked if this was
            // a data frame. Internally its represented as a 2 variant enum, but
            // that isnt part of the public interface for whatever reason.
            if let Ok(trailers) = frame.into_trailers() {
                match this.trailers {
                    Some(existing) => existing.extend(trailers),
                    None => *this.trailers = Some(trailers),
                }
            }
        }

        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        #[inline]
        const fn cast(int: u64) -> usize {
            int as usize
        }

        let Some(hint) = self.incoming.as_ref().map(Body::size_hint) else {
            return (0, Some(0));
        };

        (cast(hint.lower()), hint.upper().map(cast))
    }
}
