use crate::{
    db_handle::DbHandle,
    domain::{errors::*, Login, User},
};
use actix_files::NamedFile;
use actix_web::{get, post, web, Responder};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use snafu::ResultExt;

#[get("/")]
pub(crate) async fn login_page() -> impl Responder {
    NamedFile::open_async("./static/index.html").await
}

#[post("/")]
pub(crate) async fn login(
    input: web::Form<Login>,
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
