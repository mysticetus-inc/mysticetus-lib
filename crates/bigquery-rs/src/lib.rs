#![feature(
    trait_alias,
    slice_as_chunks,
    type_changing_struct_update,
    box_into_inner,
    seek_stream_len,
    const_trait_impl,
    let_chains,
    const_swap
)]

mod error;
pub use error::Error;

/// Type alias to [`core::result::Result<T, Error>`].
pub type Result<T> = core::result::Result<T, Error>;

// mod bindings;
mod client;
pub mod dataset;
pub mod job;
pub mod table;
pub use client::BigQueryClient;
pub mod resources;
pub mod util;

macro_rules! route {
    ($inner:expr; $($arg:expr)*) => {{
        let mut url = $inner.base_url().to_string();
        $(
            $crate::Identifier::insert_self(&$arg, &mut url);
        )+

        url
    }};
}

pub(self) use route;

/// Trait describing an identifier for a table or dataset.
///
/// This aims to somewhat merge the traits [`AsRef<str>`] and [`std::fmt::Display`],
/// that way both types + strings can be used seamlessly.
pub trait Identifier {
    /// insert the component into the path/url that's being constructed.
    fn insert_self(&self, partial_path: &mut String);
}

pub struct DisplayIdentifier<T>(pub T);

impl<T> Identifier for DisplayIdentifier<T>
where
    T: std::fmt::Display,
{
    fn insert_self(&self, partial_path: &mut String) {
        if !partial_path.ends_with('/') {
            partial_path.push_str("/");
        }
        std::fmt::Write::write_fmt(partial_path, format_args!("{}", self.0)).expect(
            "<String as fmt::Write>::write_fmt should never panic, since it's all in memory",
        )
    }
}

impl<T: Identifier + ?Sized> Identifier for &T {
    fn insert_self(&self, partial_path: &mut String) {
        T::insert_self(self, partial_path);
    }
}

impl Identifier for str {
    fn insert_self(&self, partial_path: &mut String) {
        if !partial_path.ends_with('/') {
            partial_path.push_str("/");
        }

        partial_path.push_str(self)
    }
}

static_assertions::assert_impl_all!(&str: Identifier);
static_assertions::assert_impl_all!(&&str: Identifier);

#[tokio::test]
async fn test_table_get() -> crate::Result<()> {
    let client = BigQueryClient::new(
        "mysticetus-oncloud",
        gcp_auth_channel::Scope::BigQueryReadOnly,
    )
    .await?;

    let table = client
        .dataset("oncloud_production")
        .table("geotracks")
        .get()
        .await?;

    println!("{table:#?}");

    Ok(())
}
