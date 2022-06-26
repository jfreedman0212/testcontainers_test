use actix_web::{dev::Server, get, App, HttpResponse, HttpServer};
use std::net::TcpListener;

#[get("/health_check")]
async fn greet() -> HttpResponse {
    HttpResponse::Ok().body("Application is running and ready to receive requests")
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(greet))
        .listen(listener)?
        .run();
    Ok(server)
}
