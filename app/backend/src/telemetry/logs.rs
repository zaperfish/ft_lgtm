use anyhow::Result;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};

pub fn init_logs(resource: &Resource, endpoint: &str) -> Result<SdkLoggerProvider> {
    let otlp_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let log_provider = SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(otlp_exporter)
        .build();

    Ok(log_provider)
}
