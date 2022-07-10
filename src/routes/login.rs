use crate::{
    domain::{errors::*, Login, User},
    helpers::DbHandle,
};
use actix_web::{post, web};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use snafu::ResultExt;

#[post("/login")]
pub(crate) async fn login(
    input: web::Json<Login>,
    db_handle: web::Data<DbHandle>,
) -> Result<web::Json<User>, ServerError> {
    let username = input.username().to_string();
    let user_option = db_handle
        .query_row(
            "SELECT Id, Username, PasswordHash FROM User WHERE Username = ?1",
            [username],
            |row| {
                let id: i64 = row.get(0)?;
                let username: String = row.get(1)?;
                let password_hash: String = row.get(2)?;
                Ok((User::new(id, username), password_hash))
            },
        )
        .await?;
    let argon = Argon2::default();
    match user_option {
        Some((user, password_hash)) => {
            let matches = tokio::task::spawn_blocking(move || {
                argon
                    .verify_password(
                        input.password().as_bytes(),
                        &PasswordHash::new(&password_hash).unwrap(),
                    )
                    .is_ok()
            })
            .await
            .context(JoinSnafu)?;
            if matches {
                Ok(web::Json(user))
            } else {
                Err(ServerError(InnerError::InvalidUsernameOrPassword))
            }
        }
        None => Err(ServerError(InnerError::InvalidUsernameOrPassword)),
    }
}
