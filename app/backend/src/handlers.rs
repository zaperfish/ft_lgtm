use tracing::{Span, error, field, info, warn};

use crate::app::AppState;
use crate::ipfs::Cid;
use crate::telemetry::metrics::MetricsGuard;
use crate::wasm::engine::WasmEngine;
use crate::wasm::runner::RunResult;
use crate::{ipfs, wasm};

use anyhow::Result;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Deserialize)]
struct CodeSubmission {
    language: String,
    src: String,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    run_result: RunResult,
    cid: Option<Cid>,
}

#[instrument(skip_all, fields(cid = field::Empty))]
#[handler]
pub async fn execute_handler(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<Json<RunResponse>, StatusError> {
    let state = depot.get_typed::<AppState>().unwrap();
    let mut metrics_guard = MetricsGuard::new(&state.metrics);

    let submission: CodeSubmission = req
        .parse_body()
        .await
        .map_err(|err| StatusError::bad_request().brief(err.to_string()))?;

    info!(
        source_size = submission.src.len(),
        "received code execute request"
    );

    if submission.language != "rust" {
        return Err(StatusError::bad_request().brief("unsupported language"));
    }

    let wasm_engine = WasmEngine::default();
    let run_result = wasm::runner::run(&submission.src, &wasm_engine)
        .await
        .map_err(log_and_500("failed to run src code"))?;

    let cid = match ipfs::publish(&submission.src, &run_result).await {
        Ok(cid) => Some(cid),
        Err(err) => {
            warn!(error = %err, "failed to publish to ipfs");
            None
        }
    };

    if cid.is_some() {
        if let Some(cid) = cid.as_ref() {
            Span::current().record("cid", &tracing::field::debug(cid));
        }
    }

    if run_result.succeeded() {
        info!(cid = ?cid, "code execution completed successfully");
        metrics_guard.success();
    } else {
        info!("code execution failed");
    }

    Ok(Json(RunResponse { run_result, cid }))
}

fn log_and_500<E: std::fmt::Display>(context: &'static str) -> impl FnOnce(E) -> StatusError {
    move |err| {
        error!(error = %err, "{context}");
        StatusError::internal_server_error()
    }
}
