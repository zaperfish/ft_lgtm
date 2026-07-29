use anyhow::Result;
use salvo::prelude::*;
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

    // let wasm_bin = compile_to_wasm(
    //     r#"
    //         fn main() {
    //             println!("Hello Wasm!");
    //         }
    //     "#,
    // )?;

    let router = Router::new();

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
    match program_result {
        Ok(()) => Ok("OK".to_string()),
        Err(()) => Err(anyhow::anyhow!("err")),
    }
}

fn compile_to_wasm(src: &str) -> Result<Vec<u8>> {
    let temp_dir = TempDir::new()?;
    let src_path = temp_dir.path().join("main.rs");
    let out_path = temp_dir.path().join("out.wasm");

    std::fs::write(&src_path, src)?;

    let output = std::process::Command::new("rustc")
        .args([
            "--target",
            "wasm32-wasip2",
            src_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()?;

    let wasm = std::fs::read(out_path)?;
    Ok(wasm)
}
