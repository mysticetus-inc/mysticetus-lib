use std::sync::mpsc;

use crate::subscriber::MakeWriter;

#[derive(Debug, Clone)]
pub struct MakeTestWriter<const BUFFERING: bool>(mpsc::Sender<Vec<u8>>);

impl<const BUFFERING: bool> MakeTestWriter<BUFFERING> {
    pub fn new() -> (mpsc::Receiver<Vec<u8>>, Self) {
        let (tx, rx) = mpsc::channel();
        (rx, Self(tx))
    }
}

impl<const BUFFERING: bool> MakeWriter for MakeTestWriter<BUFFERING> {
    type Writer<'a> = TestWriter;

    const NEEDS_BUFFERING: bool = BUFFERING;

    fn make_writer(&self) -> Self::Writer<'_> {
        TestWriter {
            sender: self.0.clone(),
            bytes: Vec::with_capacity(1024),
        }
    }
}

impl<'a, const BUFFERING: bool> tracing_subscriber::fmt::MakeWriter<'a>
    for MakeTestWriter<BUFFERING>
{
    type Writer = TestWriter;

    fn make_writer(&'a self) -> Self::Writer {
        MakeWriter::make_writer(self)
    }
}

pub struct TestWriter {
    sender: mpsc::Sender<Vec<u8>>,
    bytes: Vec<u8>,
}

impl std::io::Write for TestWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.bytes.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for TestWriter {
    fn drop(&mut self) {
        if !self.bytes.is_empty() {
            _ = self.sender.send(std::mem::take(&mut self.bytes));
        }
    }
}

pub struct EchoService<F>(pub F);

impl<B, F> tower::Service<http::Request<B>> for EchoService<F>
where
    F: Fn(&http::Request<B>),
{
    type Error = std::convert::Infallible;
    type Future = EchoFuture<Result<http::Response<B>, std::convert::Infallible>>;
    type Response = http::Response<B>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        (self.0)(&req);

        let (req_parts, body) = req.into_parts();

        let (mut resp_parts, body) = http::Response::builder()
            .status(http::StatusCode::OK)
            .body(body)
            .expect("should be valid")
            .into_parts();

        resp_parts.extensions = req_parts.extensions;
        resp_parts.headers = req_parts.headers;
        resp_parts.version = req_parts.version;

        resp_parts.extensions.insert(req_parts.method);
        resp_parts.extensions.insert(req_parts.uri);

        EchoFuture {
            resets: 2,
            sleep: tokio::time::sleep(tokio::time::Duration::from_millis(100)),
            ret: Some(Ok(http::Response::from_parts(resp_parts, body))),
        }
    }
}

pin_project_lite::pin_project! {
    pub struct EchoFuture<R> {
        // reset 'sleep' resets times, to imitate a future with multiple breakpoints
        resets: usize,
        #[pin]
        sleep: tokio::time::Sleep,
        ret: Option<R>,
    }
}

impl<R> Future for EchoFuture<R> {
    type Output = R;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();

        loop {
            std::task::ready!(this.sleep.as_mut().poll(cx));
            *this.resets = this.resets.checked_sub(1).unwrap();
            if *this.resets == 0 {
                break;
            }
            this.sleep
                .as_mut()
                .reset(tokio::time::Instant::now() + tokio::time::Duration::from_millis(100));
        }

        let ret = this.ret.take().expect("EchoFuture polled after completion");

        std::task::Poll::Ready(ret)
    }
}

pub struct EmptyBody;

impl http_body::Body for EmptyBody {
    type Data = bytes::Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        std::task::Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        true
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(0)
    }
}
