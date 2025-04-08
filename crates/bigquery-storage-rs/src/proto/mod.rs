//! Module that contains all of the logic needed to serialize arbitrary rows into protobufs.

mod capture;
mod encode;
// mod schemas;
mod serializer;

// We need to publicly export this module so we can fuzz test it, otherwise it stays private
#[cfg(fuzzing)]
pub use encode::zigzag;
pub(crate) use encode::{Field, FieldPair, WireType, field_type_to_wire_type};
// pub use schemas::{FieldIndex, Schemas};
#[cfg(feature = "write")]
pub use serializer::ProtoSerializer;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EncodeError {
    #[error("cannot serialize type '{0}' as a message")]
    InvalidType(&'static str),
    #[error("field '{0}' is required, but was not found during serialization")]
    MissingField(Box<str>),
    #[error("schema for field `{0}` has an unspecified data type")]
    UnspecifiedFieldType(Box<str>),
    #[error("schema for field `{0}` has an unspecified mode")]
    UnspecifiedFieldMode(Box<str>),
    #[error("{0}")]
    Misc(Box<str>),
}
