use std::net::TcpListener;

pub struct DatabaseConfiguration {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub struct ApplicationConfiguration {
    pub listener: TcpListener,
    pub database: DatabaseConfiguration,
}
