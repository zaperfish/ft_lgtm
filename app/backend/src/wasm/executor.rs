use crate::wasm::engine::WasmEngine;
use anyhow::Result;
use serde::Serialize;
use tracing::instrument;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::Command;
use wasmtime_wasi::{I32Exit, WasiCtx, WasiCtxView, WasiView};

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub status: Result<(), ExecutionError>,
    pub stdout: String,
    pub stderr: String,
}

struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    pub limits: StoreLimits,
}

#[derive(Debug, Serialize)]
pub enum ExecutionError {
    Exit(i32),
    Trap(String),
}

#[instrument(skip_all)]
pub async fn execute(bin: &[u8], wasm_engine: &WasmEngine) -> Result<ExecutionResult> {
    let engine = wasm_engine.wasmtime();
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

    let config = wasm_engine.config();

    let stdout_pipe = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(config.stdout_limit);
    let stderr_pipe = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(config.stderr_limit);

    let stdout_capture = stdout_pipe.clone();
    let stderr_capture = stderr_pipe.clone();

    let wasi = WasiCtx::builder()
        .stdout(stdout_pipe)
        .stderr(stderr_pipe)
        .build();

    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        limits: StoreLimitsBuilder::new()
            .memory_size(config.memory_limit)
            .build(),
    };

    let mut store = Store::new(engine, state);

    store.limiter(|state| &mut state.limits);

    if let Some(fuel) = config.fuel_limit {
        store.set_fuel(fuel)?;
    }

    if let Some(deadline) = config.epoch_deadline {
        store.set_epoch_deadline(deadline);
    }

    let component = Component::from_binary(engine, bin)?;
    let command = Command::instantiate_async(&mut store, &component, &linker).await?;

    let status = match command.wasi_cli_run().call_run(&mut store).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(())) => Err(ExecutionError::Exit(1)),
        Err(err) => {
            if let Some(exit) = err.downcast_ref::<I32Exit>() {
                Err(ExecutionError::Exit(exit.0))
            } else if let Some(trap) = err.downcast_ref::<Trap>() {
                Err(ExecutionError::Trap(trap.to_string()))
            } else {
                Err(ExecutionError::Exit(1))
            }
        }
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
