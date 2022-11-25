use std::{fs::File, error::Error, thread, time::Duration};

use opentelemetry::trace::FutureExt;
use tracing::instrument::WithSubscriber;
use tracing::{span, trace, warn, Level, event, info_span};
use tracing_attributes::instrument;
use tracing_subscriber::{fmt, Layer, registry::Registry};
use tracing_subscriber::prelude::*;
use tracing_opentelemetry;

use opentelemetry::{KeyValue, sdk::Resource};
use opentelemetry::{ global::shutdown_tracer_provider};

use opentelemetry_otlp::{self, WithExportConfig};

#[instrument]
#[inline]
fn expensive_work() -> &'static str {
    span!(tracing::Level::INFO, "expensive_step_1")
        .in_scope(|| thread::sleep(Duration::from_millis(25)));
    span!(tracing::Level::INFO, "expensive_step_2")
        .in_scope(|| thread::sleep(Duration::from_millis(25)));

    "success"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Then pass it into pipeline builder
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://0.0.0.0:4317");
    
    let config = opentelemetry::sdk::trace::config()
            .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name", "My Test Service 2",
                    )]));
                    
    let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .with_trace_config(config) 
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("failed to install");

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    let log_file = File::create(format!("logs/{}.log", "main"))?;
    tracing_subscriber::fmt()
        .with_writer(log_file)
        .finish()
        .with(opentelemetry)
        .init();

    // tracing_subscriber::registry()
    //     .with(opentelemetry)
    //     .try_init()?;

    {
        let root = span!(Level::INFO, "app_start", "work_units" = "2");
        let _enter = root.enter();

        let work_result = expensive_work();

        span!(Level::INFO, "faster_work")
            .in_scope(|| thread::sleep(Duration::from_millis(10)));

        info_span!("real_work2")
            .in_scope(|| thread::sleep(Duration::from_millis(10)));

        event!(Level::TRACE, "Just Trace");
        warn!("About to exit!");
        trace!("status: {}", work_result);
        println!("This is LOOOOG!");
    }

    shutdown_tracer_provider();

    Ok(())
}
