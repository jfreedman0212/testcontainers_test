use std::{fmt::Debug, path::PathBuf};

use crate::domain::errors::*;
use deadpool_sqlite::{
    rusqlite::{Error, OptionalExtension, Params, Row},
    Config, Object, Pool, Runtime,
};
use snafu::{ResultExt, Whatever};

#[derive(Clone, Debug)]
pub(crate) struct DbHandle {
    pool: Pool,
}

/// The result of `DbHandle::execute`.
pub(crate) struct ExecuteResult {
    /// The `rowid` of the last insert performed. You will usually want to use this
    /// when the operation performed was an insert. Otherwise, this should be ignored.
    pub(crate) last_insert_rowid: i64,
}

impl DbHandle {
    /// Creates a new handle to the database provided in the path.
    /// On creation, it will:
    /// 1. Create the connection pool
    /// 2. Run the necessary migrations on it
    /// 3. Set up any configuration in the database (such as journal_mode)
    ///
    /// # Parameters
    /// `path`: represents the path to the database file. Must be a valid path URL according to SQLite.
    ///
    /// # Returns
    /// `Ok(Self)` if the above operations succeed. Otherwise, an `Err(Whatever)` containing a catch-all error.
    /// With `Whatever`, there's nothing to match on, all you can do is terminate the operation you're doing.
    pub(crate) async fn from_path(path: impl Into<PathBuf>) -> Result<Self, Whatever> {
        let pool = Config::new(path)
            .create_pool(Runtime::Tokio1)
            .with_whatever_context(|error| {
                format!("Failed to create Database Connection Pool: {:?}", error)
            })?;
        Self::setup(&pool).await?;
        Ok(Self { pool })
    }

    /// Internal setup function. This mainly exists to de-dupe code in the creation functions.
    async fn setup(pool: &Pool) -> Result<(), Whatever> {
        let migration_conn = pool.get().await.with_whatever_context(|error| {
            format!(
                "Encountered error getting the migration connection from the pool: {:?}",
                error
            )
        })?;
        migration_conn
            .interact(|conn| -> Result<(), String> {
                conn.pragma_update(None, "journal_mode", &"wal")
                    .map_err(|e| {
                        format!("Failed to set pragma journal_mode=WAL because: {:?}", e)
                    })?;
                embedded::migrations::runner()
                    .run(conn)
                    .map_err(|e| format!("Failed to run database migrations because: {:?}", e))?;
                Ok(())
            })
            .await
            .with_whatever_context(|error| {
                format!("Encountered error when running `interact`: {:?}", error)
            })?
            .with_whatever_context(|e| format!("Inner error: {:?}", e))?;
        Ok(())
    }

    /// Query a single row from the database (if it exists)
    ///
    /// # Parameters
    /// * `sql`: static string containing the SQL statement to run. Must be valid SQL for SQLite to run.
    /// * `parameters`: contains any named or positional parameters to use in the SQL.
    /// * `row_mapper`: function that takes a SQL row and turns it into the `ResultType`.
    ///
    /// # Type Parameters
    /// * `ResultType`: the type contained in the returned `Option`.
    /// * `RowMapperFn`: the type of function of the `row_mapper` parameter.
    /// * `P`: the type of the parameters being used.
    ///
    /// # Returns
    /// A filled `Some(ResultType)` if a record should be returned. Otherwise, returns `None`.
    #[tracing::instrument(
        name = "Querying a single record (if it exists) from the database",
        skip(row_mapper, self, parameters)
    )]
    pub(crate) async fn query_row<ResultType, RowMapperFn, P>(
        &self,
        sql: &'static str,
        parameters: P,
        row_mapper: RowMapperFn,
    ) -> Result<Option<ResultType>, InnerError>
    where
        ResultType: Send + 'static,
        P: Params + Send + 'static + Debug,
        RowMapperFn: Send + (FnOnce(&Row<'_>) -> Result<ResultType, Error>) + 'static,
    {
        let pooled_conn = get_connection_from_pool(&self.pool).await?;
        let result = pooled_conn
            .interact(move |conn| {
                conn.prepare_cached(sql).and_then(move |mut statement| {
                    statement.query_row(parameters, row_mapper).optional()
                })
            })
            .await
            .context(DatabaseInteractSnafu)?
            .context(DatabaseConnectionSnafu)?;
        Ok(result)
    }

    /// Executes a SQL statement that modifies the database state in some way. Could be
    /// an UPDATE, a DELETE, an INSERT, etc.
    ///
    /// # Parameters
    /// * `sql`: static string containing the SQL statement to run. Must be valid SQL for SQLite to run.
    /// * `parameters`: contains any named or positional parameters to use in the SQL.
    ///
    /// # Type Parameters:
    /// * `P`: the type of the parameters being used.
    ///
    /// # Returns
    /// `Ok(ExecuteResult)` if successful (which contains information about the SQL statement), or `Err(InnerError)`
    /// if unsuccessful.
    #[tracing::instrument(
        name = "Executing a modification statement against the database",
        skip(self, parameters)
    )]
    pub(crate) async fn execute<P>(
        &self,
        sql: &'static str,
        parameters: P,
    ) -> Result<ExecuteResult, InnerError>
    where
        P: Params + Send + 'static + Debug,
    {
        let pooled_conn = get_connection_from_pool(&self.pool).await?;
        pooled_conn
            .interact(move |conn| {
                conn.prepare_cached(sql)
                    .map(|mut statement| statement.execute(parameters))?
                    .map(|_| ExecuteResult {
                        last_insert_rowid: conn.last_insert_rowid(),
                    })
            })
            .await
            .context(DatabaseInteractSnafu)?
            .context(DatabaseConnectionSnafu)
    }
}

#[tracing::instrument(
    name = "Getting a connection from the pool",
    level = "trace",
    skip(pool)
)]
async fn get_connection_from_pool(pool: &Pool) -> Result<Object, InnerError> {
    pool.get().await.context(DatabasePoolSnafu)
}

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}
