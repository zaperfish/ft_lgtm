use anyhow::Result;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};

pub fn init_traces(resource: &Resource, endpoint: &str) -> Result<SdkTracerProvider> {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(otlp_exporter)
        .build();

    Ok(tracer_provider)
}
