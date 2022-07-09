use crate::domain::{errors::ServerError, Login, User};
use actix_web::{post, web};

#[post("/login")]
pub(crate) async fn login(_input: web::Json<Login>) -> Result<web::Json<User>, ServerError> {
    todo!("Need to implement this!")
}
