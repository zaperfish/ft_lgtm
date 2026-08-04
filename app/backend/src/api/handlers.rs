use tracing::{debug, error, info, warn};

use crate::app::AppState;
use crate::ipfs;
use crate::telemetry::metrics::MetricsGuard;
use crate::wasm::compiler::{CompileResult, compile_to_wasm};
use crate::wasm::executor::{ExecutionResult, execute_wasm};

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
pub struct RunResult {
    compile_result: CompileResult,
    execution_result: Option<ExecutionResult>,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    run_result: RunResult,
    cid: Option<String>,
}

#[instrument(skip_all)]
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

    let (mut run_result, cid) = execute(&submission.src)
        .await
        .map_err(log_and_500("failed to execute"))?;

    if run_result.succeeded() {
        info!(cid = ?cid, "code execution completed successfully");
        metrics_guard.success();
    } else {
        info!("code execution failed");
    }

    run_result.compile_result.bin = None;

    Ok(Json(RunResponse { run_result, cid }))
}

async fn execute(src: &str) -> Result<(RunResult, Option<String>)> {
    let compile_result = compile_to_wasm(&src).map_err(log_and_500("failed to compile"))?;
    let execution_result = match compile_result.status {
        0 => Some(
            execute_wasm(&compile_result.bin.as_deref().unwrap())
                .await
                .map_err(log_and_500("failed to execute wasm"))?,
        ),
        _ => None,
    };

    let run_result = RunResult {
        compile_result,
        execution_result,
    };

    let cid = match ipfs::publish(&src, &run_result).await {
        Ok(cid) => Some(cid),
        Err(err) => {
            warn!(error = %err, "failed to publish result to ipfs");
            None
        }
    };

    Ok((run_result, cid))
}

fn log_and_500<E: std::fmt::Display>(context: &'static str) -> impl FnOnce(E) -> StatusError {
    move |err| {
        error!(error = %err, "{context}");
        StatusError::internal_server_error()
    }
}

impl RunResult {
    pub fn succeeded(&self) -> bool {
        self.compile_result.status == 0
            && self
                .execution_result
                .as_ref()
                .is_some_and(|result| result.status == 0)
    }
}
