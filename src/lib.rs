pub mod config;

use actix_web::{self, dev::Server, get, post, web, App, HttpResponse, HttpServer, ResponseError};
use config::ApplicationConfiguration;
use deadpool_sqlite::{rusqlite::OptionalExtension, InteractError, Pool, PoolError};
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

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct Person {
    id: i64,
    name: String,
}

impl Person {
    pub fn new(id: i64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

#[derive(Debug, Snafu)]
struct ServerError(InnerError);

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        // TODO: put more information here, such as some sort of Trace ID
        //       so I can look in the logs using that Trace ID later.
        HttpResponse::new(self.status_code())
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Snafu)]
enum InnerError {
    #[snafu(display("Failed to get a connection from the connection pool"))]
    DatabasePoolError { source: PoolError },
    #[snafu(display("Failed while connecting and running queries to the database"))]
    DatabaseConnectionError,
    #[snafu(display("Failed while running interact"))]
    DatabaseInteractError { source: InteractError },
}

#[get("/health_check")]
async fn greet() -> HttpResponse {
    HttpResponse::Ok().body("Application is running and ready to receive requests")
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
                    Person {
                        id: last_id,
                        name: person.name.clone(),
                    }
                })
        })
        .await
        .context(DatabaseInteractSnafu)?
        .map_err(|_| ServerError(InnerError::DatabaseConnectionError))?;
    Ok(web::Json(new_person))
}

#[get("/people/{id}")]
async fn get_person(
    id: web::Path<i64>,
    pool: web::Data<Pool>,
) -> Result<Option<web::Json<Person>>, ServerError> {
    let pooled_conn = pool.get().await.context(DatabasePoolSnafu)?;
    let cloned_id = *id;
    let option = pooled_conn
        .interact(move |conn| {
            conn.prepare("SELECT id, name FROM person WHERE id = ?1")
                .and_then(|mut statement| {
                    statement
                        .query_row([cloned_id], |row| {
                            Ok(Person {
                                id: row.get(0)?,
                                name: row.get(1)?,
                            })
                        })
                        .optional()
                })
        })
        .await
        .context(DatabaseInteractSnafu)?
        .map_err(|_| ServerError(InnerError::DatabaseConnectionError))?;
    Ok(option.map(web::Json))
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
            .service(greet)
            .service(create_person)
            .service(get_person)
            .app_data(web::Data::new(db_pool.clone()))
    })
    .listen(listener)
    .with_whatever_context(|error| format!("Encountered error running `listen`: {:?}", error))?
    .run())
}
