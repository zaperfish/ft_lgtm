use anyhow::Result;
use backend::app::run;
use backend::telemetry::init_telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    init_telemetry()?;
    run().await;
    Ok(())
}
