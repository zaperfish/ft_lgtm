use backend::api::run_app;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    init_tracing();
    run_app().await;
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
}
