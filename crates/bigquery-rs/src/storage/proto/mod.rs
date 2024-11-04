//! Module that contains all of the logic needed to serialize arbitrary rows into protobufs.

mod capture;
mod encode;
mod schemas;
mod serializer;

// We need to publicly export this module so we can fuzz test it, otherwise it stays private
pub(crate) use encode::FieldPair;
#[cfg(fuzzing)]
pub use encode::zigzag;
pub use schemas::{FieldIndex, Schemas};
pub use serializer::ProtoSerializer;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EncodeError {
    #[error("cannot serialize type '{0}' as a message")]
    InvalidType(&'static str),
    #[error("{0}")]
    Misc(String),
    #[error("field '{0}' is required, but was not found during serialization")]
    MissingField(String),
}
