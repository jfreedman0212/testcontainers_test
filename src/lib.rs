pub mod config;
pub mod domain;
mod helpers;
mod routes;
pub mod telemetry;

use actix_web::{dev::Server, web, App, HttpServer};
use config::ApplicationConfiguration;
use helpers::DbHandle;
use routes::{health_check, login, sign_up};
use snafu::{prelude::*, Whatever};
use tracing_actix_web::TracingLogger;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub async fn run(app_config: ApplicationConfiguration) -> Result<Server, Whatever> {
    let ApplicationConfiguration { listener, db_pool } = app_config;
    let db_handle = DbHandle::from_pool(db_pool.clone()).await?;
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(sign_up)
            .service(login)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(db_handle.clone()))
    })
    .listen(listener)
    .with_whatever_context(|error| format!("Encountered error running `listen`: {:?}", error))?
    .run())
}
