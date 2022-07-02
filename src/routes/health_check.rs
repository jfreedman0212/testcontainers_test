use actix_web::{get, HttpResponse};

#[get("/health_check")]
pub(crate) async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("Application is running and ready to receive requests")
}
