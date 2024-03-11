use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use rookie_redis::{server};
use tokio::signal::ctrl_c;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::{SubscriberInitExt, TryInitError}};

const DEFAULT_ADDR: &str = "127.0.0.1:6379";

fn init_tracing() -> Result<(), TryInitError> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            sdktrace::config()
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "rookie-redis",
                )]))
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        )
        .install_batch(runtime::Tokio)
        .expect("Unable to initialize OpenTelemetry");

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(fmt::Layer::default())
        .try_init()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing()?;
    server::run(DEFAULT_ADDR, ctrl_c()).await.unwrap();
    Ok(())
}
