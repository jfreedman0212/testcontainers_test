use deadpool_sqlite::{Config, Runtime};
use std::net::TcpListener;
use testcontainers_test::{config::ApplicationConfiguration, run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 8080))?;
    let app_config = ApplicationConfiguration {
        listener,
        db_pool: Config::new("app.sqlite3")
            .create_pool(Runtime::Tokio1)
            .unwrap(),
    };
    run(app_config).await?.await?;
    Ok(())
}
