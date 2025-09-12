pub trait MakeWriter: Send + Sync + 'static {
    type Writer<'a>: std::io::Write
    where
        Self: 'a;

    const NEEDS_BUFFERING: bool;

    const APPEND_NEWLINE: bool;

    fn make_writer(&self) -> Self::Writer<'_>;
}

pub struct StdoutWriter;

impl MakeWriter for StdoutWriter {
    type Writer<'a> = std::io::StdoutLock<'static>;

    const NEEDS_BUFFERING: bool = false;
    const APPEND_NEWLINE: bool = true;

    #[inline]
    fn make_writer(&self) -> Self::Writer<'_> {
        std::io::stdout().lock()
    }
}

pub struct NullWriter;

impl MakeWriter for NullWriter {
    type Writer<'a> = std::io::Sink;

    const NEEDS_BUFFERING: bool = false;
    const APPEND_NEWLINE: bool = false;

    fn make_writer(&self) -> Self::Writer<'_> {
        std::io::sink()
    }
}
