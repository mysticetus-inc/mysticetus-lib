use std::error::Error as StdError;

use tracing::Metadata;

use crate::Stage;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TryGetBacktrace {
    #[default]
    No,
    Yes,
    Force,
}

/// How parent span fields should be emitted in the log.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParentSpanFields {
    /// for a parent span named 'parent_span', this emits fields as a nested map, i.e:
    /// "parent_span": { "field": true },
    #[default]
    Nested,
    /// for a parent span named 'parent_span', this emits fields with a prefix
    /// consisting of the span name. i.e:
    /// "parent_span.field": true,
    Prefixed,
    /// Does no prefixing or nesting, fields are emitted as is, which may
    /// lead to duplicates if an ancestor span has an identical field name.
    Flattened,
}

pub trait LogOptions: Send + Sync + Copy + 'static {
    fn include_http_info(&self, meta: &Metadata<'_>) -> bool;

    fn treat_as_error(&self, meta: &Metadata<'_>) -> bool;

    fn include_stage(&self, stage: Stage, meta: &Metadata<'_>) -> bool;

    fn include_timestamp(&self, stage: Stage, meta: &Metadata<'_>) -> bool;

    fn parent_span_fields(&self, stage: Stage, meta: &Metadata<'_>) -> ParentSpanFields;

    fn try_get_backtrace(
        &self,
        meta: &Metadata<'_>,
        error: &(dyn StdError + 'static),
    ) -> TryGetBacktrace;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DefaultLogOptions;

impl LogOptions for DefaultLogOptions {
    fn include_http_info(&self, meta: &tracing::Metadata<'_>) -> bool {
        // include the http info on everything but verbose tracing
        !matches!(*meta.level(), tracing::Level::TRACE | tracing::Level::DEBUG)
    }

    fn treat_as_error(&self, meta: &tracing::Metadata<'_>) -> bool {
        matches!(*meta.level(), tracing::Level::ERROR)
    }

    fn include_timestamp(&self, _stage: Stage, _meta: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn include_stage(&self, _stage: Stage, _meta: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn parent_span_fields(&self, _stage: Stage, _meta: &tracing::Metadata<'_>) -> ParentSpanFields {
        ParentSpanFields::default()
    }

    fn try_get_backtrace(
        &self,
        meta: &tracing::Metadata<'_>,
        _error: &(dyn std::error::Error + 'static),
    ) -> TryGetBacktrace {
        if self.treat_as_error(meta) {
            TryGetBacktrace::Yes
        } else {
            TryGetBacktrace::No
        }
    }
}

macro_rules! impl_log_options_deref {
    ($inner:ident) => {
        #[inline]
        fn include_http_info(&self, meta: &Metadata<'_>) -> bool {
            <$inner as LogOptions>::include_http_info(&**self, meta)
        }

        #[inline]
        fn treat_as_error(&self, meta: &Metadata<'_>) -> bool {
            <$inner as LogOptions>::treat_as_error(&**self, meta)
        }

        #[inline]
        fn include_timestamp(&self, stage: Stage, meta: &Metadata<'_>) -> bool {
            <$inner as LogOptions>::include_timestamp(&**self, stage, meta)
        }

        #[inline]
        fn include_stage(&self, stage: Stage, meta: &Metadata<'_>) -> bool {
            <$inner as LogOptions>::include_stage(&**self, stage, meta)
        }

        #[inline]
        fn parent_span_fields(&self, stage: Stage, meta: &Metadata<'_>) -> ParentSpanFields {
            <$inner as LogOptions>::parent_span_fields(&**self, stage, meta)
        }

        #[inline]
        fn try_get_backtrace(
            &self,
            meta: &Metadata<'_>,
            error: &(dyn StdError + 'static),
        ) -> TryGetBacktrace {
            <$inner as LogOptions>::try_get_backtrace(&**self, meta, error)
        }
    };
}

impl<O: LogOptions> LogOptions for &'static O {
    impl_log_options_deref!(O);
}

/*
impl<O: LogOptions> LogOptions for std::sync::Arc<O> {
    impl_log_options_deref!(O);
}
*/
