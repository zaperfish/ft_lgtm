use opentelemetry::KeyValue;
use tracing::error;

use crate::app::AppState;
use crate::ipfs;
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

#[instrument(skip(req))]
#[handler]
pub async fn execute_handler(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<Json<RunResponse>, StatusError> {
    let state = depot.get_typed_mut::<AppState>().unwrap();
    let metrics = &state.metrics;

    metrics.executions_total.add(1, &[]);

    let body: CodeSubmission = req
        .parse_body()
        .await
        .map_err(|err| StatusError::bad_request().brief(err.to_string()))?;

    if body.language != "rust" {
        return Err(StatusError::bad_request().brief("unsupported language"));
    }

    let src = body.src;
    let (mut compile_result, execution_result) = compile_and_execute(&src)
        .await
        .map_err(log_and_500("failed to compile_and_execute"))?;

    compile_result.bin = None;
    let run_result = RunResult {
        compile_result,
        execution_result,
    };

    let cid = match ipfs::publish(&src, &run_result).await {
        Ok(cid) => Some(cid),
        Err(err) => {
            error!(error = %err, "failed to publish to ipfs");
            None
        }
    };

    metrics.executions_succeeded.add(1, &[]);

    Ok(Json(RunResponse { run_result, cid }))
}

fn log_and_500<E: std::fmt::Display>(context: &'static str) -> impl FnOnce(E) -> StatusError {
    move |err| {
        error!(error = %err, "{context}");
        StatusError::internal_server_error()
    }
}

async fn compile_and_execute(src: &str) -> Result<(CompileResult, Option<ExecutionResult>)> {
    let compile_result = compile_to_wasm(src)?;

    if compile_result.status != 0 {
        return Ok((compile_result, None));
    }

    let execution_result = execute_wasm(compile_result.bin.as_deref().unwrap()).await?;

    Ok((compile_result, Some(execution_result)))
}
