use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::util::TryInitError;

use super::writer::{MakeWriter, StdoutWriter};
use super::{Handle, Subscriber};
use crate::Stage;
use crate::options::{DefaultLogOptions, LogOptions};
use crate::subscriber::writer::NullWriter;

pub struct LoggingBuilder<O: LogOptions = DefaultLogOptions, MkWriter: MakeWriter = StdoutWriter> {
    pub(crate) options: O,
    pub(crate) project_id: Option<&'static str>,
    pub(crate) filter: LevelFilter,
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
            filter: tracing::level_filters::STATIC_MAX_LEVEL,
            stage,
            make_writer: StdoutWriter,
        }
    }

    #[inline]
    pub const fn new() -> Self {
        Self::new_from_stage(Stage::detect())
    }
}

impl<O, W> LoggingBuilder<O, W>
where
    O: LogOptions + 'static,
    W: MakeWriter,
{
    pub fn with_writer<MkWriter: MakeWriter>(
        self,
        make_writer: MkWriter,
    ) -> LoggingBuilder<O, MkWriter> {
        LoggingBuilder {
            options: self.options,
            filter: self.filter,
            project_id: self.project_id,
            stage: self.stage,
            make_writer,
        }
    }

    pub fn null_writer(self) -> LoggingBuilder<O, NullWriter> {
        self.with_writer(NullWriter)
    }

    pub fn with_filter<F2>(self, filter: LevelFilter) -> LoggingBuilder<O, W> {
        LoggingBuilder {
            filter,
            options: self.options,
            project_id: self.project_id,
            stage: self.stage,
            make_writer: self.make_writer,
        }
    }

    pub fn with_options<Opt2: LogOptions + Copy>(self, options: Opt2) -> LoggingBuilder<Opt2, W> {
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

    pub fn build(self) -> Subscriber<W> {
        Subscriber::from_builder(self)
    }

    #[inline]
    pub fn build_try_init(self) -> Result<Handle, TryInitError> {
        self.build().try_init()
    }

    #[inline]
    pub fn init(self) -> Handle {
        self.build().init()
    }
}
