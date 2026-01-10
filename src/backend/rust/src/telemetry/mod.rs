pub mod metrics;

use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use tracing_subscriber::prelude::*;

use crate::settings::Settings;
pub fn init_telemetry(settings: &Settings) {
    let otlp_endpoint = &settings.telemetry.otlp_endpoint;

    let resource = Resource::new(vec![
        KeyValue::new("service.name", "rust-backend"),
        KeyValue::new("deployment.environment", "development"),
    ]);

    // Configure Tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint.clone()),
        )
        .with_trace_config(sdktrace::config().with_resource(resource.clone()))
        .install_batch(runtime::Tokio)
        .expect("Failed to install tracer");

    global::set_tracer_provider(tracer.provider().unwrap());

    // Configure Metrics
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint.clone()),
        )
        .with_resource(resource.clone())
        .build()
        .expect("Failed to build metrics pipeline");

    global::set_meter_provider(meter_provider);

    // Configure Logs
    let logger = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(opentelemetry_sdk::logs::Config::default().with_resource(resource))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint.clone()),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to install logger");

    let logger_provider = logger
        .provider()
        .expect("Failed to get logger provider")
        .clone();

    // Set the global logger provider as well
    global::set_logger_provider(logger_provider.clone());

    let otel_log_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);

    // Initialize Tracing Subscriber
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("info".parse().unwrap())
        .add_directive("rust_backend=info".parse().unwrap()); // Ensure our own logs are included

    // Initialize the LogTracer to capture logs from the `log` crate and redirect them to tracing
    let _ = tracing_log::LogTracer::init();

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(telemetry)
        .with(otel_log_layer)
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    // Initialize custom metrics from the internal module
    self::metrics::MinesweeperMetrics::init();

    tracing::info!("Telemetry initialized successfully");
}
