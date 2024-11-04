use std::borrow::Cow;

// mod schema;

#[derive(Debug, Clone, PartialEq)]
pub struct InspectConfig {
    project: Cow<'static, str>,
    instance: Cow<'static, str>,
    database: Cow<'static, str>,
}

impl InspectConfig {
    /*
    async fn load(self) -> anyhow::Result<schema::InformationSchema> {
        let db = spanner_rs::info::Database::builder(self.project)
            .instance(self.instance)
            .database(self.database);

        schema::InformationSchema::read_from_db(db).await
    }
    */
}
/*
#[tokio::test]
async fn test_inspect() -> anyhow::Result<()> {
    let config = InspectConfig {
        project: "mysticetus-oncloud".into(),
        instance: "mysticetus-prod".into(),
        database: "oncloud".into(),
    };

    let schema = config.load().await?;

    println!("{schema:#?}");

    Ok(())
}
*/
