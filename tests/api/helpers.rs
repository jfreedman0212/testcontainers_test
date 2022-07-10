use once_cell::sync::Lazy;
use std::{
    env, io,
    net::{SocketAddr, TcpListener},
};
use testcontainers_test::{
    config::ApplicationConfiguration,
    run,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let name = "app";
    let env_filter = "info";
    match env::var("SHOW_LOGS") {
        Ok(val) if val == "1" => {
            let subscriber = get_subscriber(name, env_filter, io::stdout);
            init_subscriber(subscriber).unwrap();
        }
        _ => {
            let subscriber = get_subscriber(name, env_filter, io::sink);
            init_subscriber(subscriber).unwrap();
        }
    };
});

pub struct TestApp {
    pub address: SocketAddr,
}

pub async fn spawn_test_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let app_config = ApplicationConfiguration {
        listener,
        db_path: ":memory:",
    };
    let _ = tokio::spawn(async move {
        let server = run(app_config).await.unwrap();
        server.await.unwrap();
    });
    TestApp { address }
}
