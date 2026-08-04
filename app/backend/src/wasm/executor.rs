use anyhow::Result;
use serde::Serialize;
use std::sync::LazyLock;
use tracing::instrument;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::Command;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

static ENGINE: LazyLock<Engine> = LazyLock::new(|| Engine::default());

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

#[instrument(skip_all)]
pub async fn execute_wasm(bin: &[u8]) -> Result<ExecutionResult> {
    let mut linker = Linker::new(&ENGINE);
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

    let mut store = Store::new(&ENGINE, state);

    let component = Component::from_binary(&ENGINE, bin)?;
    let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    let program_result = command.wasi_cli_run().call_run(&mut store).await?;

    let status = match program_result {
        Ok(()) => 0,
        Err(()) => 1,
    };

    let stdout = String::from_utf8_lossy(&stdout_capture.contents()).into_owned();
    let stderr = String::from_utf8_lossy(&stderr_capture.contents()).into_owned();

    Ok(ExecutionResult {
        status,
        stdout,
        stderr,
    })
}

impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}
