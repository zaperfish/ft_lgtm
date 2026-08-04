use backend::app::run;
use backend::telemetry::init_telemetry;

#[tokio::main]
async fn main() {
    init_telemetry();
    run().await;
}
