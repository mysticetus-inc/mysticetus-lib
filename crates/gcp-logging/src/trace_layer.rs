use std::fmt;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use http::HeaderValue;
use tracing::Subscriber;
use tracing::span::Id;

use crate::subscriber::RequestTrace;

#[derive(Debug, Clone)]
pub struct ActiveTraces {
    active: Arc<DashMap<Id, RequestTrace, fxhash::FxBuildHasher>>,
}

impl ActiveTraces {
    pub(crate) fn new(active: Arc<DashMap<Id, RequestTrace, fxhash::FxBuildHasher>>) -> Self {
        Self { active }
    }

    pub fn get<'a>(&'a self, id: &Id) -> Option<Ref<'a, Id, RequestTrace, fxhash::FxBuildHasher>> {
        self.active.get(id)
    }

    pub fn get_first<'a, I>(
        &self,
        ids: I,
    ) -> Option<Ref<'_, Id, RequestTrace, fxhash::FxBuildHasher>>
    where
        I: IntoIterator<Item = &'a Id>,
    {
        ids.into_iter().find_map(|id| self.active.get(id))
    }
}

impl<S: Subscriber> tracing_subscriber::layer::Layer<S> for ActiveTraces {
    fn on_close(&self, id: Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        self.active.remove(&id);
    }
}

pub struct TraceHeader<'a, P = &'a str> {
    project_id: P,
    header: &'a HeaderValue,
}

impl<'a, P> TraceHeader<'a, P> {
    #[inline]
    pub fn new(project_id: P, header: &'a HeaderValue) -> Self {
        Self { project_id, header }
    }
}

impl<P: AsRef<str>> fmt::Display for TraceHeader<'_, P> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[inline]
        fn fmt_inner(
            project_id: &str,
            header: &HeaderValue,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            let trace_bytes = header.as_bytes();

            let trace = match memchr::memchr(b'/', trace_bytes) {
                Some(end_index) => &trace_bytes[..end_index],
                None => trace_bytes,
            };

            if trace.is_ascii() {
                // SAFETY: we just checked it was valid ascii. This should only ever be ascii and
                // not utf8, so we only check for ascii to avoid the extra overhead from
                // checking utf8
                let trace_str = unsafe { std::str::from_utf8_unchecked(trace) };
                write!(f, "projects/{project_id}/traces/{trace_str}")
            } else {
                // if the trace isnt ascii, something is probably wrong,
                // but try to make google listen whether they want it or not
                let trace_escaped = trace.escape_ascii();
                write!(f, "projects/{project_id}/traces/{trace_escaped}")
            }
        }

        let Self { project_id, header } = self;

        fmt_inner(project_id.as_ref(), header, f)
    }
}

impl<P: AsRef<str>> serde::Serialize for TraceHeader<'_, P> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}
