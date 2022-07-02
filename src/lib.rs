pub mod config;
pub mod domain;
mod routes;

use actix_web::{self, dev::Server, post, web, App, HttpServer};
use config::ApplicationConfiguration;
use deadpool_sqlite::Pool;
use domain::{
    errors::{DatabaseInteractSnafu, DatabasePoolSnafu, InnerError, ServerError},
    Person,
};
use routes::{get_person, health_check};
use serde::{Deserialize, Serialize};
use snafu::{prelude::*, Whatever};

#[derive(Deserialize, Serialize)]
pub struct PersonInput {
    name: String,
}

impl PersonInput {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[post("/people")]
async fn create_person(
    person: web::Json<PersonInput>,
    pool: web::Data<Pool>,
) -> Result<web::Json<Person>, ServerError> {
    let pooled_conn = pool.get().await.context(DatabasePoolSnafu)?;
    let new_person = pooled_conn
        .interact(move |conn| {
            conn.execute("INSERT INTO person (name) VALUES (?1)", [&person.name])
                .map(|_| {
                    let last_id = conn.last_insert_rowid();
                    Person::new(last_id, person.name.clone())
                })
        })
        .await
        .context(DatabaseInteractSnafu)?
        .map_err(|_| ServerError(InnerError::DatabaseConnectionError))?;
    Ok(web::Json(new_person))
}

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
            .service(health_check)
            .service(create_person)
            .service(get_person)
            .app_data(web::Data::new(db_pool.clone()))
    })
    .listen(listener)
    .with_whatever_context(|error| format!("Encountered error running `listen`: {:?}", error))?
    .run())
}
