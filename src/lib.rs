pub mod config;
mod db_handle;
pub mod domain;
mod routes;
pub mod telemetry;

use std::path::PathBuf;

use actix_web::{dev::Server, web, App, HttpServer};
use config::ApplicationConfiguration;
use db_handle::DbHandle;
use routes::{health_check, login, login_page, sign_up, sign_up_page};
use snafu::{prelude::*, Whatever};
use tracing_actix_web::TracingLogger;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub async fn run<Path: Into<PathBuf>>(
    app_config: ApplicationConfiguration<Path>,
) -> Result<Server, Whatever> {
    let ApplicationConfiguration { listener, db_path } = app_config;
    let db_handle = DbHandle::from_path(db_path).await?;
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(sign_up)
            .service(login)
            .service(login_page)
            .service(sign_up_page)
            .app_data(web::Data::new(db_handle.clone()))
    })
    .listen(listener)
    .with_whatever_context(|error| format!("Encountered error running `listen`: {:?}", error))?
    .run())
}
