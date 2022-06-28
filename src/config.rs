use deadpool_sqlite::Pool;
use std::net::TcpListener;

pub struct ApplicationConfiguration {
    pub listener: TcpListener,
    pub db_pool: Pool,
}
