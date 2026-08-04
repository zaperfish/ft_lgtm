use crate::handlers::execute_handler;
use salvo::affix_state::inject;
use salvo::prelude::*;

use crate::telemetry::metrics::Metrics;

#[derive(Debug, Clone)]
pub struct AppState {
    pub metrics: Metrics,
}

pub async fn run() {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "11000".to_string());

    let state = AppState {
        metrics: Metrics::new(),
    };

    let router = Router::with_path("api")
        .hoop(inject(state))
        .push(Router::with_path("execute").post(execute_handler));
    let service = Service::new(router);

    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(service).await;
}
