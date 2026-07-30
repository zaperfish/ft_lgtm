use crate::compiler::{CompileResult, compile_to_wasm};
use crate::executor::{ExecutionResult, execute_wasm};

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
struct CompileResponse {
    status: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Serialize)]
struct RunResponse {
    compile_result: CompileResponse,
    execution_result: Option<ExecutionResult>,
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

    let (compile_result, execution_result) = compile_and_execute(&body.src)
        .await
        .map_err(|_| StatusError::internal_server_error())?;

    Ok(Json(RunResponse {
        compile_result: compile_result.into(),
        execution_result,
    }))
}

async fn compile_and_execute(src: &str) -> Result<(CompileResult, Option<ExecutionResult>)> {
    let compile_result = compile_to_wasm(src)?;

    if compile_result.status != 0 {
        return Ok((compile_result, None));
    }

    let execution_result = execute_wasm(compile_result.bin.as_deref().unwrap()).await?;

    Ok((compile_result, Some(execution_result)))
}

impl From<CompileResult> for CompileResponse {
    fn from(result: CompileResult) -> Self {
        Self {
            status: result.status,
            stdout: result.stdout,
            stderr: result.stderr,
        }
    }
}
