use crate::domain::{
    errors::{DatabaseInteractSnafu, DatabasePoolSnafu, InnerError, ServerError},
    Person, PersonInput,
};
use actix_web::{self, post, web};
use deadpool_sqlite::Pool;
use snafu::prelude::*;
use tracing::Instrument;

#[post("/people")]
pub(crate) async fn create_person(
    person_input: web::Json<PersonInput>,
    pool: web::Data<Pool>,
) -> Result<web::Json<Person>, ServerError> {
    let request_id = uuid::Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Creating a new person",
        %request_id,
        person_name = %person_input.name()
    );
    let _span_guard = request_span.enter();
    let pool_conn_span = tracing::trace_span!("Getting a connection from the pool");
    let pooled_conn = pool
        .get()
        .instrument(pool_conn_span)
        .await
        .context(DatabasePoolSnafu)?;
    let db_span = tracing::info_span!("Inserting the person into the database");
    let new_person = pooled_conn
        .interact(move |conn| {
            conn.execute(
                "INSERT INTO person (name) VALUES (?1)",
                [person_input.name()],
            )
            .map(|_| {
                let last_id = conn.last_insert_rowid();
                Person::new(last_id, String::from(person_input.name()))
            })
        })
        .instrument(db_span)
        .await
        .context(DatabaseInteractSnafu)?
        .map_err(|_| ServerError(InnerError::DatabaseConnectionError))?;
    Ok(web::Json(new_person))
}
