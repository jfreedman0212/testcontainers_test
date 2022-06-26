use std::net::TcpListener;
use testcontainers_test::run;
use tokio;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 8080))?;
    match run(listener) {
        Ok(server) => {
            server.await?;
            Ok(())
        },
        Err(error) => Err(error),
    }
}
