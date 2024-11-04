use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::future::Then;
use tokio::signal::unix::{Signal, SignalKind, signal};

#[derive(Debug)]
pub struct Shutdown {
    /// if setting up a signal handler was successful, should a
    /// message be logged when the signal is recieved?
    log_shutdown: bool,
    /// If 'None', we weren't able to set up the signal handler,
    /// so we need to block forever.
    signal: Option<Signal>,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ShutdownWithTask<T, Fut>
where
    T: FnOnce() -> Fut,
    Fut: ShutdownTask,
{
    #[allow(dead_code)] // we transmute into this, so we never actually 'access' it
    inner: Inner<T, Fut>,
}

/// alias to not get arguments/wrapping types mixed up when
/// transmuting down in the Future impl
type Inner<T, Fut> = Then<Shutdown, Fut, LogWhenCalled<T>>;

/// For some reason [`futures::TryFuture`] was running into type resolution issues,
/// (even though they all checked out as expected). Making my own supertrait seemed
/// to solve it, so not 100% sure whats going on there.
pub trait ShutdownTask: Future<Output = Result<(), Self::Error>> {
    type Error: StdError;
}

impl<F, E> ShutdownTask for F
where
    F: Future<Output = Result<(), E>>,
    E: StdError,
{
    type Error = E;
}

fn try_setup_signal(user_defined_task: bool) -> Option<Signal> {
    match signal(SignalKind::terminate()) {
        Ok(signal) => Some(signal),
        Err(error) => {
            // if we cant set up the sigterm handler, returning early would immediately
            // shut down the server, regardless of any requests/etc.
            // to avoid this, we just need to 'never recieve the signal', i.e block here
            // forever. After 10 seconds, google will come and kill us anyways.
            if user_defined_task {
                error!(
                    message = "error setting up SIGTERM handler, shutdown task won't be run",
                    ?error
                );
            } else {
                warn!(
                    message = "error setting up SIGTERM handler, can't shut down gracefully",
                    ?error
                );
            }

            None
        }
    }
}

#[derive(Debug)]
struct LogWhenCalled<F> {
    func: F,
    log_when_called: bool,
}

impl<T, Out> FnOnce<((),)> for LogWhenCalled<T>
where
    T: FnOnce() -> Out,
{
    type Output = Out;

    extern "rust-call" fn call_once(self, _args: ((),)) -> Self::Output {
        if self.log_when_called {
            info!("starting shutdown task...");
        }
        (self.func)()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShutdownBuilder {
    log_shutdown: bool,
}

impl Default for ShutdownBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownBuilder {
    pub const fn new() -> Self {
        Self { log_shutdown: true }
    }

    pub fn dont_log_shutdown(&mut self) -> &mut Self {
        self.log_shutdown = false;
        self
    }

    pub fn listen(&mut self) -> Shutdown {
        Shutdown {
            log_shutdown: self.log_shutdown,
            signal: try_setup_signal(false),
        }
    }

    pub fn with_shutdown_task<T, Fut>(&mut self, task: T) -> ShutdownWithTask<T, Fut>
    where
        T: FnOnce() -> Fut,
        Fut: ShutdownTask,
    {
        self.listen().with_shutdown_task(task)
    }
}

impl Default for Shutdown {
    #[inline]
    fn default() -> Self {
        Self::listen()
    }
}

impl Shutdown {
    /// Default shutdown [`Future`], used in cases where no manual cleanup needs to occur.
    ///
    /// This function also defines the [`Default`] impl.
    #[inline]
    pub fn listen() -> Self {
        ShutdownBuilder::new().listen()
    }

    #[inline]
    pub const fn builder() -> ShutdownBuilder {
        ShutdownBuilder::new()
    }

    pub fn with_shutdown_task<T, Fut>(mut self, func: T) -> ShutdownWithTask<T, Fut>
    where
        T: FnOnce() -> Fut,
        Fut: ShutdownTask,
    {
        use futures::FutureExt;

        // don't log when shutdown completes, log when the task is called (if specified).
        let log_when_called = std::mem::take(&mut self.log_shutdown);

        ShutdownWithTask {
            inner: self.then(LogWhenCalled {
                func,
                log_when_called,
            }),
        }
    }
}

impl Future for Shutdown {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.signal.as_mut() {
            Some(signal) => {
                ready!(signal.poll_recv(cx));
                this.signal = None;

                if this.log_shutdown {
                    info!("SIGTERM recieved, shutting down");
                }

                Poll::Ready(())
            }
            None => Poll::Pending,
        }
    }
}

impl<T, Fut> Future for ShutdownWithTask<T, Fut>
where
    T: FnOnce() -> Fut,
    Fut: ShutdownTask,
{
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY:
        //  - ShutdownWithTask is repr(transparent)
        //  - Pin is repr(transparent)
        //  - No references are moved, so no pinning invariants are broken.
        let this: Pin<&mut Inner<T, Fut>> = unsafe { std::mem::transmute(self) };

        if let Err(error) = ready!(this.poll(cx)) {
            error!(message = "shutdown task failed", ?error);
        }

        Poll::Ready(())
    }
}
