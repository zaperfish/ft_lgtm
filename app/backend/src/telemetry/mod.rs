pub mod logs;
pub mod metrics;
pub mod traces;

use anyhow::Result;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use tracing_subscriber::{EnvFilter, prelude::*};

pub fn init_telemetry() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let resource = Resource::builder().with_service_name("backend").build();
    let endpoint = "http://ft-lgtm-observability:4317";

    logs::init_logs(&resource, endpoint)?;
    // let log_layer = OpenTelemetryTracingBridge::new(&log_provider);

    let tracer_provider = traces::init_traces(&resource, endpoint)?;
    let tracer = tracer_provider.tracer("backend");
    let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    metrics::init_metrics(&resource, endpoint)?;

    let fmt_layer = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(filter)
        // .with(log_layer)
        .with(trace_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}
