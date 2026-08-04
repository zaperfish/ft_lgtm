use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry_sdk::metrics::SdkMeterProvider;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub executions_total: Counter<u64>,
    pub executions_succeeded: Counter<u64>,
    pub executions_failed: Counter<u64>,
    pub execution_durations: Histogram<f64>,
}

impl Metrics {
    pub fn new() -> Self {
        let meter = global::meter("backend");

        Self {
            executions_total: meter.u64_counter("executions_total").build(),
            executions_succeeded: meter.u64_counter("executions_succeeded").build(),
            executions_failed: meter.u64_counter("executions_failed").build(),
            execution_durations: meter.f64_histogram("execution_durations_ms").build(),
        }
    }
}

pub fn init_metrics() {
    let meter_exporter = opentelemetry_stdout::MetricExporter::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(meter_exporter)
        .build();

    global::set_meter_provider(meter_provider);
}
