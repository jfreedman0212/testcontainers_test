use actix_web::{get, HttpResponse};

#[tracing::instrument(name = "Hitting the health check endpoint")]
#[get("/health_check")]
pub(crate) async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("Application is running and ready to receive requests")
}
