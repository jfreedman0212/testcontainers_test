pub mod config;
pub mod domain;
mod routes;
pub mod telemetry;

use actix_web::{dev::Server, web, App, HttpServer};
use config::ApplicationConfiguration;
use routes::{create_person, get_person, health_check};
use snafu::{prelude::*, Whatever};
use tracing_actix_web::TracingLogger;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub async fn run(app_config: ApplicationConfiguration) -> Result<Server, Whatever> {
    let ApplicationConfiguration { listener, db_pool } = app_config;
    // This is wrapped in a block because it ensures that the migration_conn gets dropped
    // before we run the HTTP server. When it gets dropped, it gets relinquished back to the pool
    {
        let migration_conn = db_pool.get().await.with_whatever_context(|error| {
            format!(
                "Encountered error getting the migration connection from the pool: {:?}",
                error
            )
        })?;
        migration_conn
            .interact(|conn| -> Result<(), String> {
                conn.pragma_update(None, "journal_mode", &"wal")
                    .map_err(|e| {
                        format!("Failed to set pragma journal_mode=WAL because: {:?}", e)
                    })?;
                embedded::migrations::runner()
                    .run(conn)
                    .map_err(|e| format!("Failed to run database migrations because: {:?}", e))?;
                Ok(())
            })
            .await
            .with_whatever_context(|error| {
                format!("Encountered error when running `interact`: {:?}", error)
            })?
    }
    .with_whatever_context(|e| format!("Inner error: {}", e))?;
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(create_person)
            .service(get_person)
            .app_data(web::Data::new(db_pool.clone()))
    })
    .listen(listener)
    .with_whatever_context(|error| format!("Encountered error running `listen`: {:?}", error))?
    .run())
}
