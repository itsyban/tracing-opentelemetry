use std::{error::Error, thread, time::Duration};

use opentelemetry_otlp;

use tracing::{span, trace, warn, Level};
use tracing_attributes::instrument;
use tracing_subscriber::prelude::*;
use tracing_opentelemetry;

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
    // Install an otel pipeline with a simple span processor that exports data one at a time when
    // spans end. See the `install_batch` option on each exporter's pipeline builder to see how to
    // export in batches.
    // global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    // let tracer = opentelemetry_jaeger::new_pipeline()
    //   .with_service_name("report_example")
    //   .install_simple()?;

    // Then pass it into pipeline builder

    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic();
    // Then pass it into pipeline builder
    let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .install_simple()?;

    //let tracer = stdout::new_pipeline().install_simple();
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    //let subscriber = Registry::default().with(telemetry);

    
    tracing_subscriber::registry()
        .with(opentelemetry)
        .try_init()?;

    {
        let root = span!(Level::INFO, "app_start", "work_units" = "2");
        let _enter = root.enter();

        let work_result = expensive_work();

        span!(Level::INFO, "faster_work")
            .in_scope(|| thread::sleep(Duration::from_millis(10)));

        warn!("About to exit!");
        trace!("status: {}", work_result);
    } // Once this scope is closed, all spans inside are closed as well

    // Shut down the current tracer provider. This will invoke the shutdown
    // method on all span processors. span processors should export remaining
    // spans before return.
    //global::shutdown_tracer_provider();

    Ok(())
}
