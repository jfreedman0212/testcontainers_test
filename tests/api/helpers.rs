use deadpool_sqlite::{Config, Runtime};
use once_cell::sync::Lazy;
use std::net::{SocketAddr, TcpListener};
use testcontainers_test::{
    config::ApplicationConfiguration,
    run,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("app", "info", std::io::sink);
    init_subscriber(subscriber).unwrap();
});

pub struct TestApp {
    pub address: SocketAddr,
}

pub async fn spawn_test_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let db_pool = Config::new(":memory:")
        .create_pool(Runtime::Tokio1)
        .unwrap();
    let app_config = ApplicationConfiguration { listener, db_pool };
    let _ = tokio::spawn(async move {
        let server = run(app_config).await.unwrap();
        server.await.unwrap();
    });
    TestApp { address }
}
