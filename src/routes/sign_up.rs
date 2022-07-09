use crate::domain::{errors::ServerError, User, UserInput};
use actix_web::{post, web};

#[post("/users")]
pub(crate) async fn sign_up(_input: web::Json<UserInput>) -> Result<web::Json<User>, ServerError> {
    todo!("Need to implement this!")
}
