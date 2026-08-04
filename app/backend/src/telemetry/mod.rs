pub mod logs;
pub mod metrics;
pub mod traces;

use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use tracing_subscriber::{EnvFilter, prelude::*};

pub fn init_telemetry() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let log_provider = logs::init_logs();
    let log_layer = OpenTelemetryTracingBridge::new(&log_provider);

    let tracer_provider = traces::init_traces();
    let tracer = tracer_provider.tracer("backend");
    let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    metrics::init_metrics();

    let fmt_layer = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(trace_layer)
        .with(fmt_layer)
        .init();
}
