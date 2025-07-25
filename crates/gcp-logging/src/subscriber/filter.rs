use tracing_subscriber::{Registry, layer};

use super::FormatLayer;
use super::writer::{MakeWriter, StdoutWriter};

/// Trait alias to simplify type bounds for Subscriber methods and impls,
/// without needing to expose internal types.
pub trait Filter<
    Opt: crate::LogOptions = crate::DefaultLogOptions,
    MkWriter: MakeWriter = StdoutWriter,
>: Send + Sync + layer::Layer<layer::Layered<FormatLayer<Opt, MkWriter>, Registry>>
{
}

impl<Opt, MkWriter, F> Filter<Opt, MkWriter> for F
where
    Opt: crate::LogOptions,
    MkWriter: MakeWriter,
    F: layer::Layer<layer::Layered<FormatLayer<Opt, MkWriter>, Registry>>,
    F: Send + Sync,
{
}
