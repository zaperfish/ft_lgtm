use anyhow::{Context, Result};
use salvo::prelude::*;
use serde::Deserialize;
use tempfile::TempDir;

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::Command;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "11000".to_string());

    let router = Router::new().post(run_code_handler);

    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
    Ok(())
}

pub struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RunCodeRequest {
    language: String,
    src: String,
}

#[handler]
async fn run_code_handler(req: &mut Request, res: &mut Response) {
    let run_code_req: RunCodeRequest = req.parse_body().await.unwrap();

    if run_code_req.language != "rust" {
        res.status_code = Some(StatusCode::BAD_REQUEST);
    }
    let wasm_bin = compile_to_wasm(&run_code_req.src).unwrap();
    let run_result = run_wasm(&wasm_bin).await.unwrap();
}

async fn run_wasm(wasm_bin: &[u8]) -> anyhow::Result<String> {
    let engine = Engine::default();

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

    let stdout_pipe = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(10 * 1024);

    let wasi = WasiCtx::builder().stdout(stdout_pipe.clone()).build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };

    let mut store = Store::new(&engine, state);

    let component = Component::from_binary(&engine, wasm_bin)?;
    let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    let program_result = command.wasi_cli_run().call_run(&mut store).await?;
    println!(
        "Wasm run with stdout: {}",
        String::from_utf8_lossy(&stdout_pipe.contents())
    );

    match program_result {
        Ok(()) => Ok("OK".to_string()),
        Err(()) => Err(anyhow::anyhow!("err")),
    }
}

fn compile_to_wasm(src: &str) -> Result<Vec<u8>> {
    let temp_dir = TempDir::new()?;
    let src_path = temp_dir.path().join("main.rs");
    let out_path = temp_dir.path().join("out.wasm");

    std::fs::write(&src_path, src).context("failed to write source code to file")?;

    let output = std::process::Command::new("rustc")
        .args([
            "--target",
            "wasm32-wasip2",
            src_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()?;

    println!("Status {}", output.status);

    let wasm = std::fs::read(out_path).context("failed to read wasm binary file")?;
    Ok(wasm)
}
