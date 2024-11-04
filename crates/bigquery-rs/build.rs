// const BQ_DISCOVERY_URL: &str = "https://bigquery.googleapis.com/discovery/v1/apis/bigquery/v2/rest";

const GENERATE_ENV_FLAG: &str = "GENERATE_REST_TYPES";

// use rest_discovery::CodeGenConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed={GENERATE_ENV_FLAG}");
    /*
    if std::env::var(GENERATE_ENV_FLAG).is_ok() {
        CodeGenConfig::new()
            .use_bytes()
            .format_generated_code()
            .output_file("./src/rest/bindings.rs")
            .discovery_url(BQ_DISCOVERY_URL)
            .generate()
            .await?;
    }
    */

    Ok(())
}
