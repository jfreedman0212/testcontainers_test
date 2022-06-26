use actix_web::{self, dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

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
    id: u64,
    name: String,
}

impl Person {
    pub fn new(id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub fn id(&self) -> u64 {
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
async fn create_person(person: web::Json<PersonInput>) -> impl Responder {
    web::Json(Person {
        id: 1,
        name: person.name.clone(),
    })
}

#[get("/people/{id}")]
async fn get_person(id: web::Path<u64>) -> impl Responder {
    HttpResponse::NotFound().json(Error {
        message: format!("No Person with ID {}", id),
    })
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(greet)
            .service(create_person)
            .service(get_person)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
