use std::future::{Future, IntoFuture};
use std::io;
use std::net::SocketAddr;

use futures::TryFuture;
use tokio::net::TcpListener;

#[derive(Debug, thiserror::Error)]
pub enum InitError<E: std::error::Error> {
    #[error(transparent)]
    State(E),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl<E: std::error::Error> InitError<E> {
    #[inline]
    pub fn into<E2>(self) -> E2
    where
        E2: From<io::Error> + From<E>,
    {
        match self {
            Self::Io(io) => E2::from(io),
            Self::State(state) => E2::from(state),
        }
    }

    #[inline]
    pub fn into_boxed(self) -> Box<dyn std::error::Error + Send + Sync>
    where
        Box<dyn std::error::Error + Send + Sync>: From<E>,
    {
        match self {
            Self::Io(io) => io.into(),
            Self::State(state) => Box::from(state),
        }
    }

    #[inline]
    pub fn into_boxed_local(self) -> Box<dyn std::error::Error>
    where
        Box<dyn std::error::Error>: From<E>,
    {
        match self {
            Self::Io(io) => io.into(),
            Self::State(state) => Box::from(state),
        }
    }

    #[cfg(feature = "anyhow")]
    #[inline]
    pub fn into_anyhow(self) -> anyhow::Error
    where
        anyhow::Error: From<E>,
    {
        self.into::<anyhow::Error>()
    }
}

pub async fn init_listener_and_state<Fut>(
    initialize_state_future: impl IntoFuture<IntoFuture = Fut>,
) -> Result<(SocketAddr, TcpListener, Fut::Ok), InitError<Fut::Error>>
where
    Fut: TryFuture + Future<Output = Result<Fut::Ok, Fut::Error>>,
    Fut::Error: std::error::Error,
{
    // use an inner future so we can inspect the result to log errors
    // without explicitly needing to in external user-code
    let inner_fut = async move {
        use futures::future::{Either, try_select};

        let addr = crate::addr();

        let listener_future = std::pin::pin!(TcpListener::bind(addr));
        let state_future = std::pin::pin!(initialize_state_future.into_future());

        match try_select(listener_future, state_future).await {
            Ok(Either::Left((listener, state_fut))) => {
                let state = state_fut.await.map_err(InitError::State)?;
                Ok((addr, listener, state))
            }
            Ok(Either::Right((state, listener_fut))) => {
                let listener = listener_fut.await.map_err(InitError::Io)?;
                Ok((addr, listener, state))
            }
            Err(Either::Left((error, _))) => Err(InitError::Io(error)),
            Err(Either::Right((error, _))) => Err(InitError::State(error)),
        }
    };

    match inner_fut.await {
        Ok(parts) => Ok(parts),
        Err(InitError::Io(io)) => {
            tracing::error!(
                message="error initializing listener",
                error.debug = ?io,
                error.display = %io,
                alert=true,
            );
            Err(InitError::Io(io))
        }
        Err(InitError::State(state)) => {
            tracing::error!(
                message="error initializing state",
                error.debug = ?state,
                error.display = %state,
                alert=true,
            );
            Err(InitError::State(state))
        }
    }
}
