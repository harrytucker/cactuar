use color_eyre::Result;
use tracing::{subscriber::set_global_default, Level, Subscriber};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::filter_fn, prelude::*, EnvFilter, Layer};

/// Tokio Console requires that the [`tokio`] and `runtime` targets be logged at
/// the TRACE level. This constant is used to add a directive to [`EnvFilter`]
/// to do this.
const TOKIO_CONSOLE_FILTERS: &str = "tokio=trace";
const RUNTIME_CONSOLE_FILTERS: &str = "runtime=trace";

/// Returns a [`tracing`] subscriber to receive structured logging events.
///
/// To set this as the global logger, as well as to receive events from the
/// standard library log facade, call [`set_global_logger`].
pub fn new_subscriber() -> Result<impl Subscriber + Send + Sync> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive(TOKIO_CONSOLE_FILTERS.parse()?)
        .add_directive(RUNTIME_CONSOLE_FILTERS.parse()?);

    // Enable support for capturing span traces when errors occur, used for
    // error reports with the `color-eyre` crate.
    let span_errors = ErrorLayer::default();

    // Enables Tokio Console support for debugging, filters out TRACE level
    // events by default in order avoid burying application logs in Tokio TRACE
    // events.
    let tokio_console = console_subscriber::spawn();
    let tokio_filter = filter_fn(|metadata| {
        metadata.level() != &Level::TRACE && metadata.target() != "tokio"
            || metadata.level() != &Level::TRACE && metadata.target() != "runtime"
    });

    // automatically switch between pretty and json log formats depending on the
    // compilation profile
    //
    // the output needs to boxed in order to erase the interior types to get the
    // match arm to function, see the following:
    // https://docs.rs/tracing-subscriber/0.3.11/tracing_subscriber/layer/trait.Layer.html#method.boxed
    let log_format = match cfg!(debug_assertions) {
        true => tracing_subscriber::fmt::layer()
            .pretty()
            .with_filter(tokio_filter)
            .boxed(),
        false => tracing_subscriber::fmt::layer()
            .json()
            .with_filter(tokio_filter)
            .boxed(),
    };

    Ok(tracing_subscriber::registry()
        .with(tokio_console)
        .with(env_filter)
        .with(log_format)
        .with(span_errors))
}

/// Initialises [`LogTracer`] to capture log events with [`tracing`], and sets
/// the given subscriber as the global default subscriber for structured logging
/// events. Also enables [`color_eyre`] error and panic handling hooks for
/// developer happiness.
///
/// Calling this twice will result in a code panic.
pub fn install_observability(subscriber: impl Subscriber + Send + Sync) -> Result<()> {
    color_eyre::install()?;
    LogTracer::init()?;
    set_global_default(subscriber)?;

    Ok(())
}
