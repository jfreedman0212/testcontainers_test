use crate::{
    domain::{errors::*, User, UserInput},
    db_handle::{DbHandle, ExecuteResult},
};
use actix_web::{post, web};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use snafu::ResultExt;

#[tracing::instrument(name = "Creating a user", skip(input, db_handle), fields(username = %input.username()))]
#[post("/users")]
pub(crate) async fn sign_up(
    input: web::Json<UserInput>,
    db_handle: web::Data<DbHandle>,
) -> Result<web::Json<User>, ServerError> {
    let password_hash = hash_and_salt_password(input.password().into()).await?;
    let ExecuteResult { last_insert_rowid } = db_handle
        .execute(
            "INSERT INTO User (Username, PasswordHash) VALUES (?1, ?2)",
            [input.username().to_string(), password_hash],
        )
        .await?;
    Ok(web::Json(User::new(
        last_insert_rowid,
        input.username().into(),
    )))
}

#[tracing::instrument(name = "Hashing and salting a password", skip(password))]
async fn hash_and_salt_password(password: String) -> Result<String, InnerError> {
    tokio::task::spawn_blocking(move || {
        let argon = Argon2::default(); // TODO: may want to use a custom instance
        let salt = SaltString::generate(&mut OsRng);
        argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error_source| InnerError::PasswordHashingError { error_source })
            .map(|hashed_password| hashed_password.to_string())
    })
    .await
    .context(JoinSnafu)?
}
