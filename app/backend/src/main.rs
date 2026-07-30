use tracing_subscriber::fmt::format::FmtSpan;

mod app;

#[tokio::main]
async fn main() {
    init_tracing();
    app::run_app().await;
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
}
