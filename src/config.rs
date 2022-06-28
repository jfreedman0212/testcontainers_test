use std::net::TcpListener;
use deadpool_sqlite::Config;

pub struct DatabaseConfiguration {
    pub config: Config
}

pub struct ApplicationConfiguration {
    pub listener: TcpListener,
    pub database: DatabaseConfiguration,
}
