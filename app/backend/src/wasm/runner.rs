use crate::wasm::compiler::{CompileResult, compile};
use crate::wasm::engine::WasmEngine;
use crate::wasm::executor::{ExecutionResult, execute};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RunResult {
    compile_result: CompileResult,
    execution_result: Option<ExecutionResult>,
}

pub async fn run(src: &str, wasm_engine: &WasmEngine) -> Result<RunResult> {
    let compile_result = compile(&src).await?;

    let execution_result = match compile_result.status {
        0 => Some(execute(&compile_result.bin.as_deref().unwrap(), wasm_engine).await?),
        _ => None,
    };

    Ok(RunResult {
        compile_result,
        execution_result,
    })
}

impl RunResult {
    pub fn succeeded(&self) -> bool {
        self.compile_result.status == 0
            && self
                .execution_result
                .as_ref()
                .is_some_and(|result| result.status.is_ok())
    }
}
