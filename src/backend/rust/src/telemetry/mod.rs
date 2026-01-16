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
        KeyValue::new("deployment.environment", settings.environment.clone()),
    ]);

    let mut telemetry_layer = None;
    let mut otel_log_layer = None;

    if !otlp_endpoint.is_empty() {
        // Configure Tracer
        match opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otlp_endpoint.clone()),
            )
            .with_trace_config(sdktrace::config().with_resource(resource.clone()))
            .install_batch(runtime::Tokio)
        {
            Ok(tracer) => {
                if let Some(provider) = tracer.provider() {
                    global::set_tracer_provider(provider.clone());
                } else {
                    eprintln!("Tracer installed without provider; traces will not be exported");
                }
                telemetry_layer = Some(tracing_opentelemetry::layer().with_tracer(tracer));
            }
            Err(e) => eprintln!("Failed to install tracer: {}", e),
        }

        // Configure Metrics
        match opentelemetry_otlp::new_pipeline()
            .metrics(runtime::Tokio)
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otlp_endpoint.clone()),
            )
            .with_resource(resource.clone())
            .build()
        {
            Ok(meter_provider) => global::set_meter_provider(meter_provider),
            Err(e) => eprintln!("Failed to build metrics pipeline: {}", e),
        }

        // Configure Logs
        match opentelemetry_otlp::new_pipeline()
            .logging()
            .with_log_config(opentelemetry_sdk::logs::Config::default().with_resource(resource))
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otlp_endpoint.clone()),
            )
            .install_batch(runtime::Tokio)
        {
            Ok(logger) => {
                if let Some(logger_provider) = logger.provider() {
                    let logger_provider = logger_provider.clone();
                    global::set_logger_provider(logger_provider.clone());
                    otel_log_layer = Some(
                        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                            &logger_provider,
                        ),
                    );
                }
            }
            Err(e) => eprintln!("Failed to install logger: {}", e),
        }
    }

    // Initialize Tracing Subscriber
    let mut filter = tracing_subscriber::EnvFilter::from_default_env();
    for directive in ["info", "rust_backend=info"] {
        match directive.parse() {
            Ok(d) => filter = filter.add_directive(d),
            Err(e) => eprintln!("Invalid log directive '{directive}': {e}"),
        }
    }

    // Initialize the LogTracer to capture logs from the `log` crate and redirect them to tracing
    let _ = tracing_log::LogTracer::init();

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(telemetry_layer)
        .with(otel_log_layer)
        .with(tracing_subscriber::fmt::layer());

    if let Err(e) = registry.try_init() {
        eprintln!("Failed to initialize tracing registry: {}", e);
    }

    // Initialize custom metrics from the internal module
    self::metrics::MinesweeperMetrics::init();

    tracing::info!("Telemetry initialized successfully");
}
