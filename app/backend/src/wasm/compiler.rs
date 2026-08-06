use anyhow::{Context, Result};
use serde::Serialize;
use tempfile::TempDir;
use tracing::instrument;

#[derive(Debug, Serialize)]
pub struct CompileResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    #[serde(skip_serializing)]
    pub bin: Option<Vec<u8>>,
}

#[instrument(skip_all)]
pub async fn compile_to_wasm(src: &str) -> Result<CompileResult> {
    let temp_dir = TempDir::new()?;
    let src_path = temp_dir.path().join("main.rs");
    let out_path = temp_dir.path().join("out.wasm");

    std::fs::write(&src_path, src).context("failed to write source code to file")?;

    let output = tokio::process::Command::new("timeout")
        .arg("5s")
        .arg("rustc")
        .arg("--target")
        .arg("wasm32-wasip2")
        .arg(&src_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .await?;

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
