spanner_rs::table! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct TestTable {
        #[spanner(pk = 1)]
        test_field: usize,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn emulator_setup() -> spanner_rs::Result<()> {
    let db = spanner_rs::Database::builder("mysticetus-oncloud")
        .instance("mysticetus-dev")
        .database("mysticetus-templates");

    let remote_client = db
        .build_client(gcp_auth_channel::Scope::SpannerAdmin)
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

    Ok(())
}
