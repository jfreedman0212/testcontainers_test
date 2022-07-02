use snafu::{ResultExt, Whatever};
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: impl Into<String>,
    env_filter: impl Into<String>,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter.into()));
    let formatting_layer = BunyanFormattingLayer::new(name.into(), sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) -> Result<(), Whatever> {
    LogTracer::init()
        .with_whatever_context(|e| format!("Failed to init the LogTracer: {:?}", e))?;
    set_global_default(subscriber).with_whatever_context(|e| {
        format!("Could not set the global Tracing subscriber: {:?}", e)
    })?;
    Ok(())
}
