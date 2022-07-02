use actix_web::{get, HttpResponse};

#[get("/health_check")]
pub(crate) async fn health_check() -> HttpResponse {
    let request_id = uuid::Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Hitting the health check endpoint",
        %request_id
    );
    let _span_guard = request_span.enter();
    HttpResponse::Ok().body("Application is running and ready to receive requests")
}
