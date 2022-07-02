pub mod config;
pub mod domain;
mod routes;

use actix_web::{self, dev::Server, middleware::Logger, web, App, HttpServer};
use config::ApplicationConfiguration;
use routes::{create_person, get_person, health_check};
use snafu::{prelude::*, Whatever};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub async fn run(app_config: ApplicationConfiguration) -> Result<Server, Whatever> {
    let ApplicationConfiguration { listener, db_pool } = app_config;
    let migration_conn = db_pool.get().await.with_whatever_context(|error| {
        format!(
            "Encountered error getting the migration connection from the pool: {:?}",
            error
        )
    })?;
    migration_conn
        .interact(|conn| {
            // it's okay to panic in here because `interact` will catch it.
            // otherwise, we want to propagate errors through Result.
            // additionally, this happens during application startup, so panicking is okay
            embedded::migrations::runner().run(conn).unwrap();
        })
        .await
        .with_whatever_context(|error| {
            format!(
                "Encountered error running the database migration: {:?}",
                error
            )
        })?;
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(health_check)
            .service(create_person)
            .service(get_person)
            .app_data(web::Data::new(db_pool.clone()))
    })
    .listen(listener)
    .with_whatever_context(|error| format!("Encountered error running `listen`: {:?}", error))?
    .run())
}
