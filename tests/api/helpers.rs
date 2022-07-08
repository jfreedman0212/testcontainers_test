use deadpool_sqlite::{Config, Runtime};
use once_cell::sync::Lazy;
use std::{
    env, io,
    net::{SocketAddr, TcpListener},
};
use testcontainers::{clients::Cli, core::WaitFor, images::generic::GenericImage, Container};
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

pub struct TestApp<'a> {
    pub address: SocketAddr,
    _container: Container<'a, GenericImage>,
}

pub fn spawn_test_app<'a>(docker: &'a Cli) -> TestApp<'a> {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let db = "postgres-db-test";
    let user = "postgres-user-test";
    let password = "postgres-password-test";
    let port = 5432;
    let postgres = GenericImage::new("postgres", "14.4-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", db)
        .with_env_var("POSTGRES_USER", user)
        .with_env_var("POSTGRES_PASSWORD", password)
        .with_exposed_port(port);
    let container = docker.run(postgres);
    let db_pool = Config::new(":memory:")
        .create_pool(Runtime::Tokio1)
        .unwrap();
    let app_config = ApplicationConfiguration { listener, db_pool };
    let _ = tokio::spawn(async move {
        let server = run(app_config).await.unwrap();
        server.await.unwrap();
    });
    TestApp {
        address,
        _container: container,
    }
}
