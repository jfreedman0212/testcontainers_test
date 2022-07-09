use crate::domain::{errors::*, Login, User};
use actix_web::{post, web};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use deadpool_sqlite::{
    rusqlite::{named_params, OptionalExtension},
    Pool,
};
use snafu::ResultExt;
use tracing::Instrument;

#[post("/login")]
pub(crate) async fn login(
    input: web::Json<Login>,
    pool: web::Data<Pool>,
) -> Result<web::Json<User>, ServerError> {
    let pool_conn_span = tracing::trace_span!("Getting a connection from the pool");
    let pooled_conn = pool
        .get()
        .instrument(pool_conn_span)
        .await
        .context(DatabasePoolSnafu)?;
    let db_span = tracing::info_span!("Retrieving the user from the database");
    let username: String = input.username().into();
    let (user, _) = pooled_conn
        .interact(move |conn| {
            conn.prepare_cached(
                "SELECT Id, Username, PasswordHash FROM User WHERE Username = :username",
            )
            .and_then(|mut statement| {
                statement
                    .query_row(named_params! { ":username": username }, |row| {
                        let id: i64 = row.get(0)?;
                        let username: String = row.get(1)?;
                        let password_hash: String = row.get(2)?;
                        Ok((User::new(id, username), password_hash))
                    })
                    .optional()
            })
        })
        .instrument(db_span)
        .await
        .context(DatabaseInteractSnafu)?
        .context(DatabaseConnectionSnafu)?
        .filter(|(_, password_hash)| {
            let argon = Argon2::default();
            argon
                .verify_password(
                    input.password().as_bytes(),
                    &PasswordHash::new(password_hash).unwrap(),
                )
                .is_ok()
        })
        .ok_or(InnerError::InvalidUsernameOrPassword)?;
    Ok(web::Json(user))
}
