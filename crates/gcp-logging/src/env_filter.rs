use std::ops::Deref;
use std::sync::OnceLock;

use tracing::level_filters::LevelFilter;
use tracing::span::{Attributes, Id, Record};
use tracing::subscriber::Interest;
use tracing::{Event, Metadata};
use tracing_subscriber::layer::{Context, Filter};

pub struct EnvFilter {
    level_filter: OnceLock<LevelFilter>,
    fallback_level_filter: LevelFilter,
}

impl EnvFilter {
    pub fn new() -> Self {
        Self::new_with_fallback(LevelFilter::INFO)
    }

    pub fn new_with_fallback(fallback_level_filter: LevelFilter) -> Self {
        Self {
            fallback_level_filter,
            level_filter: OnceLock::new(),
        }
    }

    #[inline]
    pub fn get(&self) -> &LevelFilter {
        fn get_env_level() -> Option<LevelFilter> {
            let var = std::env::var("RUST_LOG").ok()?;
            var.parse().ok()
        }

        self.level_filter
            .get_or_init(|| get_env_level().unwrap_or(self.fallback_level_filter))
    }
}

impl Deref for EnvFilter {
    type Target = LevelFilter;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<S> Filter<S> for EnvFilter {
    #[inline]
    fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool {
        <LevelFilter as Filter<S>>::enabled(self.get(), meta, cx)
    }

    #[inline]
    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        <LevelFilter as Filter<S>>::callsite_enabled(self.get(), meta)
    }

    #[inline]
    fn event_enabled(&self, event: &Event<'_>, cx: &Context<'_, S>) -> bool {
        <LevelFilter as Filter<S>>::event_enabled(self.get(), event, cx)
    }

    #[inline]
    fn max_level_hint(&self) -> Option<LevelFilter> {
        <LevelFilter as Filter<S>>::max_level_hint(self.get())
    }

    #[inline]
    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        <LevelFilter as Filter<S>>::on_record(self.get(), id, values, ctx)
    }

    #[inline]
    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        <LevelFilter as Filter<S>>::on_enter(self.get(), id, ctx)
    }

    #[inline]
    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        <LevelFilter as Filter<S>>::on_close(self.get(), id, ctx)
    }

    #[inline]
    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        <LevelFilter as Filter<S>>::on_exit(self.get(), id, ctx)
    }

    #[inline]
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        <LevelFilter as Filter<S>>::on_new_span(self.get(), attrs, id, ctx)
    }
}

impl From<LevelFilter> for EnvFilter {
    fn from(value: LevelFilter) -> Self {
        Self {
            fallback_level_filter: value,
            level_filter: OnceLock::from(value),
        }
    }
}
