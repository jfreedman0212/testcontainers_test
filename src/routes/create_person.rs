use crate::domain::{
    errors::{DatabaseInteractSnafu, DatabasePoolSnafu, InnerError, ServerError},
    Person, PersonInput,
};
use actix_web::{self, post, web};
use deadpool_sqlite::Pool;
use snafu::prelude::*;

#[post("/people")]
pub(crate) async fn create_person(
    person_input: web::Json<PersonInput>,
    pool: web::Data<Pool>,
) -> Result<web::Json<Person>, ServerError> {
    let pooled_conn = pool.get().await.context(DatabasePoolSnafu)?;
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
        .await
        .context(DatabaseInteractSnafu)?
        .map_err(|_| ServerError(InnerError::DatabaseConnectionError))?;
    Ok(web::Json(new_person))
}
