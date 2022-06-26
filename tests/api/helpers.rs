use std::net::{SocketAddr, TcpListener};
use testcontainers::{clients::Http, core::WaitFor, images};
use testcontainers_test::{
    config::{ApplicationConfiguration, DatabaseConfiguration},
    run,
};
use tokio_postgres::{connect, NoTls};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub struct TestApp {
    pub address: SocketAddr,
}

pub async fn spawn_test_app() -> TestApp {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let db = "postgres-db-test";
    let user = "postgres-user-test";
    let password = "postgres-password-test";
    let port = 5432;
    let docker = Http::default();
    let postgres = images::generic::GenericImage::new("postgres", "14.4-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", db)
        .with_env_var("POSTGRES_USER", user)
        .with_env_var("POSTGRES_PASSWORD", password)
        .with_exposed_port(port);
    let node = docker.run(postgres).await;
    let app_config = ApplicationConfiguration {
        listener,
        database: DatabaseConfiguration {
            name: String::from(db),
            host: String::from("127.0.0.1"),
            port: node.get_host_port_ipv4(port).await,
            username: String::from(user),
            password: String::from(password),
        },
    };
    let (mut client, _) = connect(
        format!(
            "host={} port={} user={} password={} dbname={}",
            app_config.database.host,
            app_config.database.port,
            app_config.database.username,
            app_config.database.password,
            app_config.database.name
        )
        .as_str(),
        NoTls,
    )
    .await
    .unwrap();
    embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .unwrap();
    let _ = tokio::spawn(async move {
        run(app_config).unwrap().await.unwrap();
    });
    TestApp { address }
}
