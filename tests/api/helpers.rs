use deadpool_sqlite::{Config, Runtime};
use std::net::{SocketAddr, TcpListener};
use testcontainers_test::{config::ApplicationConfiguration, run};

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
    let db_pool = Config::new(":memory:")
        .create_pool(Runtime::Tokio1)
        .unwrap();
    let migration_conn = db_pool.get().await.unwrap();
    let app_config = ApplicationConfiguration { listener, db_pool };
    migration_conn
        .interact(|conn| {
            embedded::migrations::runner().run(conn).unwrap();
        })
        .await
        .unwrap();
    let _ = tokio::spawn(async move {
        run(app_config).await.unwrap().await.unwrap();
    });
    TestApp { address }
}
