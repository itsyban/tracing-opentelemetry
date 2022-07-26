use std::{error::Error, thread, time::Duration};

use tracing::{span, trace, warn, Level, event, info_span};
use tracing_attributes::instrument;
use tracing_subscriber::prelude::*;
use tracing_opentelemetry;

use opentelemetry;
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
    
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://0.0.0.0:4317");
    // Then pass it into pipeline builder
    
    let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter) 
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("failed to install");

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(opentelemetry)
        .try_init()?;

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
    }

    shutdown_tracer_provider();

    Ok(())
}
