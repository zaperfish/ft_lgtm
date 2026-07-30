use salvo::prelude::*;
use tracing_subscriber::fmt::format::FmtSpan;

mod runner;

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

async fn run_app() {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "11000".to_string());

    let router = Router::with_path("api").push(runner::get_run_routes());

    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
}
