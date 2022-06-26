use std::net::TcpListener;

use actix_web::{dev::Server, get, web, App, HttpServer, Responder};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(greet))
        .listen(listener)?
        .run();
    Ok(server)
}
