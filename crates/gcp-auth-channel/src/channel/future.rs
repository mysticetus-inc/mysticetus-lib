use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use http::{Request, header};
use pin_project_lite::pin_project;
use tower::Service;

use super::{AuthChannel, Error};
use crate::auth::RefreshHeaderFuture;

pin_project! {
    pub struct AuthFuture<Svc, Body>
    where
        Svc: Service<Request<Body>>,
        Svc::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        pub(super) channel: AuthChannel<Svc>,
        #[pin]
        pub(super) state: State<Svc, Body>,
    }
}

pin_project_lite::pin_project! {
    #[project = StateProject]
    #[project_replace = StateReplace]
    pub enum State<Svc, Body>
    where
        Svc: Service<Request<Body>>,
        Svc::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        GettingToken {
            #[pin]
            future: RefreshHeaderFuture,
            req: Option<Request<Body>>,
        },
        MakingRequest {
            #[pin]
            future: <Svc as Service<Request<Body>>>::Future,
        }
    }
}

fn poll_channel_fut<Svc, Body>(
    fut: Pin<&mut Svc::Future>,
    cx: &mut Context<'_>,
) -> Poll<Result<Svc::Response, Error>>
where
    Svc: Service<Request<Body>>,
    Svc::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    fut.poll(cx).map_err(|e| Error::Transport(e.into()))
}

impl<Svc, Body> Future for AuthFuture<Svc, Body>
where
    Svc: Service<Request<Body>>,
    Svc::Future: Unpin,
    Svc::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Output = Result<Svc::Response, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use StateProject::*;

        let mut this = self.project();

        match this.state.as_mut().project() {
            GettingToken { future, req } => match ready!(future.poll(cx)) {
                Ok(header) => {
                    let mut req = req.take().expect("invalid state");

                    req.headers_mut().insert(header::AUTHORIZATION, header);

                    let mut future = Service::call(&mut this.channel.svc, req);
                    // I don't imagine the call will be ready immediately, but we should
                    // set the state regardless, since that's the state it's left in
                    let poll_res = poll_channel_fut::<Svc, Body>(Pin::new(&mut future), cx);
                    this.state.set(State::MakingRequest { future });
                    poll_res
                }
                Err(error) => Poll::Ready(Err(error)),
            },
            MakingRequest { future } => poll_channel_fut::<Svc, Body>(future, cx),
        }
    }
}
