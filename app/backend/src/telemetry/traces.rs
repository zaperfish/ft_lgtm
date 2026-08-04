use opentelemetry_sdk::trace::SdkTracerProvider;

pub fn init_traces() -> SdkTracerProvider {
    let trace_exporter = opentelemetry_stdout::SpanExporter::default();
    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(trace_exporter)
        .build();

    tracer_provider
}
