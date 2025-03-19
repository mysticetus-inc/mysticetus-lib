#![feature(
    const_trait_impl,
    exact_size_is_empty,
    extend_one,
    maybe_uninit_write_slice,
    int_roundings,
    mapped_lock_guards
)]
#[macro_use]
extern crate tracing;

mod batch_write;
mod client;
pub mod column;
pub mod convert;
pub mod error;
pub mod info;
pub mod insertable;
pub mod key_set;
#[doc(hidden)]
pub mod macros;
pub mod pk;
pub mod queryable;
pub mod results;
pub mod serde;
// mod session;
pub mod sql;
pub mod table;
pub mod tx;
pub mod ty;
mod util;
mod value;
pub mod with;

#[cfg(feature = "admin")]
pub use client::admin;
#[cfg(feature = "emulator")]
pub use client::emulator;
pub use client::{Client, SessionClient};
pub use convert::{FromSpanner, IntoSpanner, SpannerEncode};
pub use error::Error;
pub use info::Database;
pub use pk::{PartialPkParts, PkParts, PrimaryKey};
pub use results::{ResultIter, StreamingRead};
// pub use session::Session;
pub use table::Table;
pub use ty::{Field, Scalar, Type};
pub use value::{Row, Value};

#[doc(hidden)] // for `spanner-rs-macros`
pub mod __private {
    pub use protos::{protobuf, spanner};
}

/// Trait representing the different types of 'connections' to Spanner.
///
/// Used to abstract bare [`Session`]s and different transaction types, while
/// providing the same interface between them all.
pub trait ReadableConnection:
    for<'a, 'sess> private::SealedConnection<'sess, Tx<'a>: tx::ReadOnlyTx>
{
}

/// Blanket impl over the real implementors.
impl<T> ReadableConnection for T where
    for<'a, 'sess> T: private::SealedConnection<'sess, Tx<'a>: tx::ReadOnlyTx>
{
}

#[doc(hidden)]
pub mod __macro_internals {
    // re-export for macro usage
    pub use {generic_array, static_casing, typenum};

    use crate::Field;
    use crate::convert::{FromSpanner, SpannerEncode};
    use crate::error::ConvertError;

    #[inline]
    #[doc(hidden)]
    pub fn from_spanner<T: FromSpanner>(
        field: &Field,
        value: crate::Value,
    ) -> Result<T, ConvertError> {
        FromSpanner::from_field_and_value(field, value)
    }

    #[inline]
    #[doc(hidden)]
    pub fn to_spanner<T: SpannerEncode>(value: T) -> Result<crate::Value, ConvertError> {
        SpannerEncode::encode_to_value(value).map_err(Into::into)
    }
}

/// Internal-only module for sealed traits.
mod private {
    use protos::protobuf::ListValue;

    use crate::client::connection::ConnectionParts;

    // pub trait Sealed {}

    /// Sealed trait for primary keys (or partial primary keys). Implemented only for tuples,
    /// with special impls for tuples last N types being () as a marker for an unpopulated key
    /// component.
    pub trait SealedToKey {
        /// Convert to the protobuf list that the spanner interface is expecting.
        fn to_key(self) -> ListValue;
    }

    // Spanner itself limits primary keys to 16 columns
    spanner_rs_macros::impl_pk_sealed!(SealedToKey; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

    /// Trait representing types that can hand out a [`Session`], and interact with spanner data.
    ///
    /// Used mainly to treat bare [`Session`]'s + the different transaction types opaquely
    pub trait SealedConnection<'session> {
        /// The type of transaction that 'self' implicitely contains into (by default).
        type Tx<'a>: SealedTx
        where
            Self: 'a;

        fn connection_parts(&self) -> ConnectionParts<'_, 'session, Self::Tx<'_>>;
    }

    /// Base transaction trait. Really only used as a bound for the supertraits to keep implementors
    /// sealed in this crate. ([`ReadOnlyTx`] and [`ReadWriteTx`])
    ///
    /// [`ReadOnlyTx`]: crate::tx::ReadOnlyTx
    /// [`ReadWriteTx`]: crate::tx::ReadWriteTx
    pub trait SealedTx: Copy {}
}

/// Type alias for [`core::result::Result`] with the error variant pre-set to [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
