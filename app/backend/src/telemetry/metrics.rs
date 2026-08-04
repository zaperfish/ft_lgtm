use anyhow::Result;
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::time::Instant;

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

pub struct MetricsGuard<'a> {
    metrics: &'a Metrics,
    start: Instant,
    success: bool,
}

impl<'a> MetricsGuard<'a> {
    pub fn new(metrics: &'a Metrics) -> Self {
        Self {
            metrics,
            start: Instant::now(),
            success: false,
        }
    }

    pub fn success(self: &mut Self) {
        self.success = true;
    }
}

impl Drop for MetricsGuard<'_> {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_millis();
        self.metrics.executions_total.add(1, &[]);

        if self.success {
            self.metrics.executions_succeeded.add(1, &[]);
            self.metrics
                .execution_durations
                .record(duration as f64, &[]);
        } else {
            self.metrics.executions_failed.add(1, &[]);
        }
    }
}

pub fn init_metrics(resource: &Resource, endpoint: &str) -> Result<()> {
    let otlp_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_periodic_exporter(otlp_exporter)
        .build();

    global::set_meter_provider(meter_provider);
    Ok(())
}
