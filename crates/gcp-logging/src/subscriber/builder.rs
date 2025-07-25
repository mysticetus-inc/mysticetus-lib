use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::util::TryInitError;

use super::writer::{MakeWriter, StdoutWriter};
use super::{Filter, Handle, Subscriber};
use crate::Stage;
use crate::options::{DefaultLogOptions, LogOptions};

pub struct LoggingBuilder<
    O: LogOptions = DefaultLogOptions,
    F = LevelFilter,
    MkWriter: MakeWriter = StdoutWriter,
> {
    pub(crate) options: O,
    pub(crate) project_id: Option<&'static str>,
    pub(crate) filter: F,
    pub(crate) stage: Stage,
    pub(crate) make_writer: MkWriter,
}

impl Default for LoggingBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingBuilder {
    #[inline]
    pub const fn new_from_stage(stage: Stage) -> Self {
        Self {
            options: DefaultLogOptions,
            project_id: None,
            filter: LevelFilter::INFO,
            stage,
            make_writer: StdoutWriter,
        }
    }

    #[inline]
    pub const fn new() -> Self {
        #[cfg(test)]
        let stage = Stage::Test;
        #[cfg(all(not(test), debug_assertions))]
        let stage = Stage::Dev;
        #[cfg(all(not(test), not(debug_assertions)))]
        let stage = Stage::Production;

        Self::new_from_stage(stage)
    }
}

impl<O, W, F> LoggingBuilder<O, F, W>
where
    O: LogOptions,
    W: MakeWriter,
    F: Filter<O, W>,
{
    pub fn with_writer<MkWriter: MakeWriter>(
        self,
        make_writer: MkWriter,
    ) -> LoggingBuilder<O, F, MkWriter>
    where
        F: Filter<O, MkWriter>,
    {
        LoggingBuilder {
            options: self.options,
            filter: self.filter,
            project_id: self.project_id,
            stage: self.stage,
            make_writer,
        }
    }

    pub fn with_filter<F2>(self, filter: F2) -> LoggingBuilder<O, F2, W>
    where
        F2: Filter<O, W>,
    {
        LoggingBuilder {
            filter,
            options: self.options,
            project_id: self.project_id,
            stage: self.stage,
            make_writer: self.make_writer,
        }
    }

    pub fn with_options<Opt2: LogOptions>(self, options: Opt2) -> LoggingBuilder<Opt2, F, W>
    where
        F: Filter<Opt2, W>,
    {
        LoggingBuilder {
            options,
            project_id: self.project_id,
            filter: self.filter,
            stage: self.stage,
            make_writer: self.make_writer,
        }
    }

    pub fn project_id(mut self, project_id: impl Into<&'static str>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn build(self) -> Subscriber<O, F, W> {
        Subscriber::from_builder(self)
    }

    #[inline]
    pub fn build_try_init(self) -> Result<Handle<O>, TryInitError> {
        self.build().try_init()
    }

    #[inline]
    pub fn init(self) -> Handle<O> {
        self.build().init()
    }
}
