pub mod config;

use actix_web::{self, dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};
use config::ApplicationConfiguration;
use deadpool_sqlite::{rusqlite::OptionalExtension, Pool, Runtime};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct Error {
    message: String,
}

#[get("/health_check")]
async fn greet() -> HttpResponse {
    HttpResponse::Ok().body("Application is running and ready to receive requests")
}

#[post("/people")]
async fn create_person(person: web::Json<PersonInput>, pool: web::Data<Pool>) -> impl Responder {
    let pooled_conn = pool.get().await.unwrap(); // TODO: do not unwrap!
    let new_person = pooled_conn
        .interact(move |conn| {
            let _ = conn.execute("INSERT INTO person (name) VALUES (?1)", [&person.name]);
            let last_id = conn.last_insert_rowid();
            Person {
                id: last_id,
                name: person.name.clone(),
            }
        })
        .await
        .unwrap(); // TODO: do not unwrap!
    web::Json(new_person)
}

#[get("/people/{id}")]
async fn get_person(id: web::Path<i64>, pool: web::Data<Pool>) -> impl Responder {
    let pooled_conn = pool.get().await.unwrap(); // TODO: do not unwrap!
    let cloned_id = id.clone();
    let option = pooled_conn
        .interact(move |conn| {
            let mut statement = conn
                .prepare("SELECT id, name FROM person WHERE id = ?1")
                .unwrap();
            statement
                .query_row([cloned_id], |row| {
                    Ok(Person {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                })
                .optional()
        })
        .await
        .unwrap()
        .unwrap();
    match option {
        Some(person) => HttpResponse::Ok().json(person),
        None => HttpResponse::NotFound().json(Error {
            message: format!("No Person with ID {}", id),
        }),
    }
}

pub async fn run(app_config: ApplicationConfiguration) -> Result<Server, std::io::Error> {
    let ApplicationConfiguration { listener, database } = app_config;
    let pool: Pool = database.config.create_pool(Runtime::Tokio1).unwrap(); // TODO: map the error to something
    let server = HttpServer::new(move || {
        App::new()
            .service(greet)
            .service(create_person)
            .service(get_person)
            .app_data(web::Data::new(pool.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
