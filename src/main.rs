use deadpool_sqlite::Config;
use std::net::TcpListener;
use testcontainers_test::{
    config::{ApplicationConfiguration, DatabaseConfiguration},
    run,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 8080))?;
    let app_config = ApplicationConfiguration {
        listener,
        database: DatabaseConfiguration {
            config: Config::new("app.sqlite3"),
        },
    };
    match run(app_config).await {
        Ok(server) => {
            server.await?;
            Ok(())
        }
        Err(error) => Err(error),
    }
}
