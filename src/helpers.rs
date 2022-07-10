use crate::domain::errors::*;
use deadpool_sqlite::{Object, Pool};
use snafu::ResultExt;

#[tracing::instrument(
    name = "Getting a connection from the pool",
    level = "trace",
    skip(pool)
)]
pub(crate) async fn get_connection_from_pool(pool: &Pool) -> Result<Object, InnerError> {
    pool.get().await.context(DatabasePoolSnafu)
}
