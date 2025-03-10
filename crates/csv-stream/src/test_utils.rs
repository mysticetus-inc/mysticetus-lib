use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, Bytes};
use rand::Rng;
use rand::rngs::ThreadRng;

const YIELD_MIN: usize = 1;
const YIELD_MAX: usize = 20;

const MIN_WAIT_MS: u64 = 50;
const MAX_WAIT_MS: u64 = 250;

pin_project_lite::pin_project! {
    /// A horrible stream of bytes meant to find edge cases
    /// by inserting random sleeps, and only yielding a
    /// small and random number of bytes at a time
    pub(crate) struct HorribleStream {
        data: Bytes,
        rng: rand::rngs::ThreadRng,
        #[pin]
        state: StreamState,
    }
}

impl HorribleStream {
    pub(crate) async fn from_file<P: AsRef<Path> + ?Sized>(path: &P) -> std::io::Result<Self> {
        let data = tokio::fs::read(path.as_ref()).await?;
        Ok(Self::new(Bytes::from(data)))
    }

    pub(crate) fn new(data: Bytes) -> Self {
        let mut rng = rand::rng();
        let state = StreamState::generate(&mut rng);

        Self { data, rng, state }
    }
}

pin_project_lite::pin_project! {
    #[project = StreamStateProjection]
    enum StreamState {
        Waiting {
            #[pin] sleep: tokio::time::Sleep,
        },
        Reading,
    }
}

impl StreamState {
    fn generate(rng: &mut impl Rng) -> Self {
        if rng.random() {
            Self::Reading
        } else {
            let duration_ms = rng.random_range(MIN_WAIT_MS..MAX_WAIT_MS);
            let duration = tokio::time::Duration::from_millis(duration_ms);
            Self::Waiting {
                sleep: tokio::time::sleep(duration),
            }
        }
    }
}

impl futures::Stream for HorribleStream {
    type Item = Result<Bytes, std::convert::Infallible>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match this.state.as_mut().project() {
                StreamStateProjection::Reading => {
                    if this.data.is_empty() {
                        return Poll::Ready(None);
                    }

                    let to_read = this
                        .rng
                        .random_range(YIELD_MIN..YIELD_MAX)
                        .min(this.data.len());

                    let data = this.data.copy_to_bytes(to_read);

                    // randomly set the state for next iteration
                    this.state.set(StreamState::generate(this.rng));

                    return Poll::Ready(Some(Ok(data)));
                }
                StreamStateProjection::Waiting { sleep } => {
                    std::task::ready!(sleep.poll(cx));
                    this.state.set(StreamState::Reading);
                }
            }
        }
    }
}

pub struct HorribleBuf {
    state: RefCell<(ThreadRng, HorribleBufState)>,
    bytes: Bytes,
}

enum HorribleBufState {
    NextLen(NonZeroUsize, Option<NonZeroUsize>),
    Random,
}

impl HorribleBuf {
    pub(crate) fn new(bytes: impl Into<Bytes>) -> Self {
        Self {
            state: RefCell::new((rand::rng(), HorribleBufState::Random)),
            bytes: bytes.into(),
        }
    }
}

impl Buf for HorribleBuf {
    fn remaining(&self) -> usize {
        self.bytes.remaining()
    }

    fn advance(&mut self, cnt: usize) {
        self.bytes.advance(cnt);
    }

    fn chunk(&self) -> &[u8] {
        if self.bytes.is_empty() {
            return &[];
        }
        let mut ref_mut = self.state.borrow_mut();

        let (rng, state) = &mut *ref_mut;

        let to_take = match *state {
            HorribleBufState::NextLen(len, next_len) => {
                *state = match next_len {
                    None => HorribleBufState::Random,
                    Some(next_len) => HorribleBufState::NextLen(next_len, None),
                };
                len
            }
            HorribleBufState::Random => {
                let len = NonZeroUsize::new(rng.random_range(1..=self.bytes.len())).unwrap();
                let repeated_len = if rng.random() { Some(len) } else { None };
                *state = HorribleBufState::NextLen(len, repeated_len);
                len
            }
        };

        match self.bytes.get(..to_take.get()) {
            Some(chunk) => chunk,
            None => {
                *state = HorribleBufState::Random;
                _ = drop(ref_mut);
                self.chunk()
            }
        }
    }
}
