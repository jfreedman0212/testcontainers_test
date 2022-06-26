use std::net::TcpListener;
use testcontainers_test::{
    config::{ApplicationConfiguration, DatabaseConfiguration},
    run,
};
use tokio;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 8080))?;
    let app_config = ApplicationConfiguration {
        listener,
        database: DatabaseConfiguration {
            name: String::from("test_database"),
            host: String::from("127.0.0.1"),
            port: 5432,
            username: String::from("dummy"),
            password: String::from("qwe123QWE!@#"),
        },
    };
    match run(app_config) {
        Ok(server) => {
            server.await?;
            Ok(())
        }
        Err(error) => Err(error),
    }
}
