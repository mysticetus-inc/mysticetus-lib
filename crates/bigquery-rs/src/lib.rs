mod error;
pub use error::Error;

/// Type alias to [`core::result::Result<T, Error>`].
pub type Result<T> = core::result::Result<T, Error>;

mod client;
pub mod dataset;
pub mod job;
pub mod table;
pub use client::BigQueryClient;
pub mod resources;
pub mod util;

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
