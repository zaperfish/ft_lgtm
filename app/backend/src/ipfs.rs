use crate::handlers::RunResult;
use anyhow::{Result, anyhow};
use reqwest::multipart::Part;
use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Deserialize)]
struct IpfsItem {
    #[serde(rename = "Hash")]
    hash: String,
}

#[instrument(skip_all)]
pub async fn publish(src: &str, run_result: &RunResult) -> Result<String> {
    const URL: &str = "http://ft-lgtm-ipfs-node:5001/api/v0/add?wrap-with-directory=true";

    let src_part = Part::text(src.to_string())
        .file_name("main.rs")
        .mime_str("text/plain")?;

    let result_part = Part::text(serde_json::to_string(run_result)?)
        .file_name("run_result.json")
        .mime_str("application/json")?;

    let form = reqwest::multipart::Form::new()
        .part("file", src_part)
        .part("file", result_part);

    let client = reqwest::Client::new();
    let res = client.post(URL).multipart(form).send().await?;
    res.error_for_status_ref()?;

    let body = res.text().await?;
    let cid = body
        .lines()
        .last()
        .map(serde_json::from_str::<IpfsItem>)
        .transpose()?
        .ok_or_else(|| anyhow!("no IPFS response"))?
        .hash;

    Ok(cid)
}
