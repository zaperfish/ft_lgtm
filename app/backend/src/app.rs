use anyhow::{Context, Result};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tracing::instrument;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::Command;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

pub struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

#[derive(Debug, Deserialize)]
struct RunCodeRequest {
    language: String,
    src: String,
}

#[derive(Debug, Serialize)]
struct CompileResult {
    status: i32,
    stdout: String,
    stderr: String,
    bin: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
struct CompileResponse {
    status: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Serialize)]
struct RunResult {
    status: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Serialize)]
struct RunResponse {
    compile_result: CompileResponse,
    run_result: Option<RunResult>,
}

pub async fn run_app() {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "11000".to_string());

    let router = Router::with_path("api").push(Router::with_path("code/run").post(run_handler));

    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
}

#[instrument(skip(req))]
#[handler]
async fn run_handler(req: &mut Request) -> Result<Json<RunResponse>, StatusError> {
    let body: RunCodeRequest = req
        .parse_body()
        .await
        .map_err(|err| StatusError::bad_request().brief(err.to_string()))?;

    if body.language != "rust" {
        return Err(StatusError::bad_request().brief("unsupported language"));
    }

    let response = compile_and_run(&body.src)
        .await
        .map_err(|_| StatusError::internal_server_error())?;

    Ok(Json(response))
}

async fn compile_and_run(src: &str) -> Result<RunResponse> {
    let compile_result = compile_to_wasm(src)?;

    if compile_result.status != 0 {
        return Ok(RunResponse {
            compile_result: compile_result.into(),
            run_result: None,
        });
    }

    let run_result = run_wasm(compile_result.bin.as_deref().unwrap()).await?;

    Ok(RunResponse {
        compile_result: compile_result.into(),
        run_result: Some(run_result),
    })
}

#[instrument(skip(bin))]
async fn run_wasm(bin: &[u8]) -> Result<RunResult> {
    let engine = Engine::default();

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

    let stdout_pipe = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(10 * 1024);
    let stderr_pipe = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(10 * 1024);

    let stdout_capture = stdout_pipe.clone();
    let stderr_capture = stderr_pipe.clone();

    let wasi = WasiCtx::builder()
        .stdout(stdout_pipe)
        .stderr(stderr_pipe)
        .build();

    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };

    let mut store = Store::new(&engine, state);

    let component = Component::from_binary(&engine, bin)?;
    let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    let program_result = command.wasi_cli_run().call_run(&mut store).await?;

    let status = match program_result {
        Ok(()) => 0,
        Err(()) => 1,
    };

    let stdout = String::from_utf8_lossy(&stdout_capture.contents()).into_owned();
    let stderr = String::from_utf8_lossy(&stderr_capture.contents()).into_owned();

    Ok(RunResult {
        status,
        stdout,
        stderr,
    })
}

#[instrument(skip(src))]
fn compile_to_wasm(src: &str) -> Result<CompileResult> {
    let temp_dir = TempDir::new()?;
    let src_path = temp_dir.path().join("main.rs");
    let out_path = temp_dir.path().join("out.wasm");

    std::fs::write(&src_path, src).context("failed to write source code to file")?;

    let output = std::process::Command::new("rustc")
        .arg("--target")
        .arg("wasm32-wasip2")
        .arg(&src_path)
        .arg("-o")
        .arg(&out_path)
        .output()?;

    let bin = if output.status.success() {
        Some(std::fs::read(out_path).context("failed to read wasm binary file")?)
    } else {
        None
    };

    let status = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(CompileResult {
        status,
        stdout,
        stderr,
        bin,
    })
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

impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}
