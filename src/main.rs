use deadpool_sqlite::{Config, Runtime};
use snafu::{prelude::*, Whatever};
use std::net::TcpListener;
use testcontainers_test::{config::ApplicationConfiguration, run};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), Whatever> {
    LogTracer::init()
        .with_whatever_context(|e| format!("Failed to init the LogTracer: {:?}", e))?;
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("app".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).with_whatever_context(|e| {
        format!("Could not set the global Tracing subscriber: {:?}", e)
    })?;
    let listener = TcpListener::bind(("127.0.0.1", 8080))
        .with_whatever_context(|error| format!("Failed to create TCP Listener: {:?}", error))?;
    let db_pool = Config::new("app.sqlite3")
        .create_pool(Runtime::Tokio1)
        .with_whatever_context(|error| {
            format!("Failed to create Database Connection Pool: {:?}", error)
        })?;
    run(ApplicationConfiguration { listener, db_pool })
        .await?
        .await
        .with_whatever_context(|error| {
            format!(
                "Failed to start application/failed while running: {:?}",
                error
            )
        })?;
    Ok(())
}
