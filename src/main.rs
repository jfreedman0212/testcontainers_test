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
    run(ApplicationConfiguration {
        listener,
        db_path: "app.sqlite3",
    })
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
