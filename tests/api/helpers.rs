use deadpool_sqlite::{rusqlite, Config};
use std::net::{SocketAddr, TcpListener};
use testcontainers_test::{
    config::{ApplicationConfiguration, DatabaseConfiguration},
    run,
};
use uuid::Uuid;

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
    let db_path = format!("{}.integration.sqlite3", Uuid::new_v4());
    let app_config = ApplicationConfiguration {
        listener,
        database: DatabaseConfiguration {
            config: Config::new(db_path.clone()),
        },
    };
    let _ = tokio::task::spawn_blocking(|| {
        embedded::migrations::runner()
            .run(&mut rusqlite::Connection::open(db_path).unwrap())
            .unwrap();
    })
    .await;
    let _ = tokio::spawn(async move {
        run(app_config).await.unwrap().await.unwrap();
    });
    TestApp { address }
}
