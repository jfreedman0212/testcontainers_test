use crate::domain::{
    errors::{DatabaseConnectionSnafu, DatabaseInteractSnafu, DatabasePoolSnafu, ServerError},
    Person, PersonInput,
};
use actix_web::{post, web};
use deadpool_sqlite::{rusqlite::named_params, Pool};
use snafu::prelude::*;
use tracing::Instrument;

#[tracing::instrument(
    name = "Creating a new person",
    skip(person_input, pool),
    fields(person_name = %person_input.name())
)]
#[post("/people")]
pub(crate) async fn create_person(
    person_input: web::Json<PersonInput>,
    pool: web::Data<Pool>,
) -> Result<web::Json<Person>, ServerError> {
    let pool_conn_span = tracing::trace_span!("Getting a connection from the pool");
    let pooled_conn = pool
        .get()
        .instrument(pool_conn_span)
        .await
        .context(DatabasePoolSnafu)?;
    let db_span = tracing::info_span!("Inserting the person into the database");
    let new_person = pooled_conn
        .interact(move |conn| {
            conn.prepare_cached("INSERT INTO person (name) VALUES (:name)")
                .map(|mut statement| {
                    statement.execute(named_params! { ":name": person_input.name() })
                })?
                .map(|_| {
                    let last_id = conn.last_insert_rowid();
                    Person::new(last_id, String::from(person_input.name()))
                })
        })
        .instrument(db_span)
        .await
        .context(DatabaseInteractSnafu)?
        .context(DatabaseConnectionSnafu)?;
    Ok(web::Json(new_person))
}
