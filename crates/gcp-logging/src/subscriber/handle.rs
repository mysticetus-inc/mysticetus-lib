use std::sync::{Arc, Weak};

use http_body::Body;

use super::Shared;
use crate::http_request::RequestTrace;
use crate::records::ActiveRequest;
use crate::{DefaultLogOptions, LogOptions};

/// Handle to a constructed [Subscriber].
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Handle<O: LogOptions = DefaultLogOptions> {
    pub(super) shared: Arc<Shared<O>>,
}

impl<O: LogOptions> Handle<O> {
    pub fn downgrade(&self) -> WeakHandle<O> {
        WeakHandle {
            shared: Arc::downgrade(&self.shared),
        }
    }

    fn try_get_current_inner<Filter, MkWriter>() -> Option<Self>
    where
        Filter: super::Filter<O, MkWriter>,
        MkWriter: super::MakeWriter,
    {
        tracing::dispatcher::get_default(|dispatcher| {
            dispatcher
                .downcast_ref::<super::Subscriber<O, Filter, MkWriter>>()
                .map(|subscriber| subscriber.handle())
        })
    }

    /// Attempts to get a handle to the currently active subscriber.
    ///
    /// That could mean a scoped one, or the global one, if set. Might fail
    /// even if one is set, if the actual current subscriber is using a different
    /// set of generics for the options, filter and writer.
    ///
    /// This will only succeed if the currently active subscriber is [Subscriber<O>];
    pub fn try_get_current() -> Option<Self> {
        #[cfg(not(test))]
        type MkWriter = super::StdoutWriter;
        #[cfg(test)]
        type MkWriter = crate::test_utils::MakeTestWriter<false>;

        // defers to the default generic types on Subscriber (or with MakeTestWriter
        // if we're running in cfg(test))
        Self::try_get_current_inner::<tracing_core::LevelFilter, MkWriter>()
    }

    pub fn start_request_for_span<B: Body>(
        &self,
        span: &tracing::Span,
        request: &http::Request<B>,
    ) -> Option<ActiveRequest> {
        let id = span.id()?;
        let meta = span.metadata()?;

        let trace = RequestTrace::new(request);

        let active_request =
            self.shared
                .records
                .start_new_request(id, meta, None, trace, self.shared.options);

        Some(active_request)
    }

    pub fn set_project_id(&self, project_id: &'static str) -> Result<(), ()> {
        self.shared.project_id.set(project_id).map_err(|_| ())
    }

    pub fn get_or_init_project_id(
        &self,
        get_project_id: impl FnOnce() -> &'static str,
    ) -> &'static str {
        self.shared.project_id.get_or_init(get_project_id)
    }
}

/// Weak handle to a constructed [Subscriber].
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct WeakHandle<O: LogOptions = DefaultLogOptions> {
    pub(super) shared: Weak<Shared<O>>,
}

impl<O: LogOptions> WeakHandle<O> {
    pub fn upgrade(&self) -> Option<Handle<O>> {
        self.shared.upgrade().map(|shared| Handle { shared })
    }
}
