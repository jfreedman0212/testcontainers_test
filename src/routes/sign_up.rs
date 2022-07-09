use crate::domain::{errors::*, User, UserInput};
use actix_web::{post, web};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use deadpool_sqlite::{rusqlite::named_params, Pool};
use snafu::ResultExt;
use tracing::Instrument;

#[tracing::instrument(name = "Creating a user", skip(input, pool), fields(username = %input.username()))]
#[post("/users")]
pub(crate) async fn sign_up(
    input: web::Json<UserInput>,
    pool: web::Data<Pool>,
) -> Result<web::Json<User>, ServerError> {
    let pool_conn_span = tracing::trace_span!("Getting a connection from the pool");
    let pooled_conn = pool
        .get()
        .instrument(pool_conn_span)
        .await
        .context(DatabasePoolSnafu)?;
    let password_hash = hash_and_salt_password(input.password().into()).await?;
    let db_span = tracing::info_span!("Inserting the new person into the database");
    let new_user = pooled_conn
        .interact(move |conn| {
            conn.prepare_cached(
                "INSERT INTO User (Username, PasswordHash) VALUES (:username, :password_hash)",
            )
            .map(|mut statement| {
                statement.execute(named_params! {
                    ":username": input.username(),
                    ":password_hash": password_hash
                })
            })?
            .map(|_| {
                let last_id = conn.last_insert_rowid();
                User::new(last_id, input.username().into())
            })
        })
        .instrument(db_span)
        .await
        .context(DatabaseInteractSnafu)?
        .context(DatabaseConnectionSnafu)?;
    Ok(web::Json(new_user))
}

#[tracing::instrument(name = "Hashing and salting the password", skip(password))]
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
