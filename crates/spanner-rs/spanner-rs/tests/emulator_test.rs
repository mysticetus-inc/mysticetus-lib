#[cfg(not(all(feature = "emulator", feature = "admin")))]
use spanner_rs::Database;

spanner_rs::row! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[spanner(table = "TestTable")]
    pub struct TestTable {
        #[spanner(pk = 1)]
        pub test_field: usize,
    }
}

#[cfg_attr(
    not(all(feature = "emulator", feature = "admin")),
    ignore = "requires both the 'emulator' and 'admin' features to be enabled"
)]
#[tokio::test(flavor = "current_thread")]
async fn emulator_setup() -> spanner_rs::Result<()> {
    let db = spanner_rs::Database::builder("mysticetus-oncloud")
        .instance("mysticetus-dev")
        .database("mysticetus-templates");

    run(db).await
}

#[cfg(not(all(feature = "emulator", feature = "admin")))]
async fn run(_database: Database) -> spanner_rs::Result<()> {
    Ok(())
}

#[cfg(all(feature = "emulator", feature = "admin"))]
async fn run(database: Database) -> spanner_rs::Result<()> {
    let remote_client = database
        .build_client(gcp_auth_provider::Scope::SpannerAdmin)
        .await?;

    let emulator_opts = spanner_rs::emulator::EmulatorOptions::default();
    let compute = spanner_rs::admin::InstanceCompute::Nodes(1);

    let (emulator, emulator_client) = remote_client
        .replicate_db_setup_emulator(emulator_opts, compute)
        .await?;

    let mut session = emulator_client.create_session().await?;

    let mut results = session
        .execute_sql::<TestTable>("SELECT 1 AS TestField".to_owned(), None)
        .await?;

    let row = results.next().unwrap()?;

    assert_eq!(row.test_field, 1);
}
