use deadpool_sqlite::{Config, Runtime};
use snafu::{prelude::*, Whatever};
use std::net::TcpListener;
use testcontainers_test::{
    config::ApplicationConfiguration,
    run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), Whatever> {
    let subscriber = get_subscriber("app", "info", std::io::stdout);
    init_subscriber(subscriber)?;
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
