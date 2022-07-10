use std::{net::TcpListener, path::PathBuf};

pub struct ApplicationConfiguration<Path: Into<PathBuf>> {
    pub listener: TcpListener,
    pub db_path: Path,
}
