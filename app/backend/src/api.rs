use tracing::error;

use crate::compiler::{CompileResult, compile_to_wasm};
use crate::executor::{ExecutionResult, execute_wasm};
use crate::ipfs;

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
    cid: String,
}

pub async fn run_app() {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "11000".to_string());

    let router = Router::with_path("api").push(run_router());

    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
}

fn run_router() -> Router {
    Router::with_path("code/run").post(run_handler)
}

#[instrument(skip(req))]
#[handler]
async fn run_handler(req: &mut Request) -> Result<Json<RunResponse>, StatusError> {
    let body: CodeSubmission = req
        .parse_body()
        .await
        .map_err(|err| StatusError::bad_request().brief(err.to_string()))?;

    if body.language != "rust" {
        return Err(StatusError::bad_request().brief("unsupported language"));
    }

    let src = body.src;
    let (mut compile_result, execution_result) =
        compile_and_execute(&src).await.map_err(|err| {
            error!(error = %err, "compile_and_exexute failed");
            StatusError::internal_server_error()
        })?;

    compile_result.bin = None;
    let run_result = RunResult {
        compile_result,
        execution_result,
    };

    let cid = ipfs::publish(&src, &run_result).await.map_err(|err| {
        error!(error = %err, "publishing to ipfs node failed");
        StatusError::internal_server_error()
    })?;

    Ok(Json(RunResponse { run_result, cid }))
}

async fn compile_and_execute(src: &str) -> Result<(CompileResult, Option<ExecutionResult>)> {
    let compile_result = compile_to_wasm(src)?;

    if compile_result.status != 0 {
        return Ok((compile_result, None));
    }

    let execution_result = execute_wasm(compile_result.bin.as_deref().unwrap()).await?;

    Ok((compile_result, Some(execution_result)))
}
