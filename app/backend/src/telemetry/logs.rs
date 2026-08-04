use opentelemetry_sdk::logs::SdkLoggerProvider;

pub fn init_logs() -> SdkLoggerProvider {
    let log_exporter = opentelemetry_stdout::LogExporter::default();
    let log_provider = SdkLoggerProvider::builder()
        .with_simple_exporter(log_exporter)
        .build();

    log_provider
}
