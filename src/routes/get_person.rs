use crate::domain::{
    errors::{DatabaseInteractSnafu, DatabasePoolSnafu, InnerError, ServerError},
    Person,
};
use actix_web::{get, web};
use deadpool_sqlite::{rusqlite::OptionalExtension, Pool};
use snafu::prelude::*;
use tracing::Instrument;

#[tracing::instrument(name = "Fetching a person", skip(pool))]
#[get("/people/{id}")]
pub(crate) async fn get_person(
    id: web::Path<i64>,
    pool: web::Data<Pool>,
) -> Result<Option<web::Json<Person>>, ServerError> {
    let pool_conn_span = tracing::trace_span!("Getting a connection from the pool");
    let pooled_conn = pool
        .get()
        .instrument(pool_conn_span)
        .await
        .context(DatabasePoolSnafu)?;
    let cloned_id = *id;
    let db_span = tracing::info_span!("Retrieving the person from the database");
    let option = pooled_conn
        .interact(move |conn| {
            conn.prepare("SELECT id, name FROM person WHERE id = ?1")
                .and_then(|mut statement| {
                    statement
                        .query_row([cloned_id], |row| Ok(Person::new(row.get(0)?, row.get(1)?)))
                        .optional()
                })
        })
        .instrument(db_span)
        .await
        .context(DatabaseInteractSnafu)?
        .map_err(|_| ServerError(InnerError::DatabaseConnectionError))?;
    Ok(option.map(web::Json))
}
