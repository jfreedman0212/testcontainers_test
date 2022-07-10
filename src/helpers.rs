use std::{fmt::Debug, path::PathBuf};

use crate::domain::errors::*;
use deadpool_sqlite::{
    rusqlite::{Error, OptionalExtension, Params, Row},
    Config, Object, Pool, Runtime,
};
use snafu::{ResultExt, Whatever};

#[tracing::instrument(
    name = "Getting a connection from the pool",
    level = "trace",
    skip(pool)
)]
pub(crate) async fn get_connection_from_pool(pool: &Pool) -> Result<Object, InnerError> {
    pool.get().await.context(DatabasePoolSnafu)
}

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

#[derive(Clone, Debug)]
pub(crate) struct DbHandle {
    pool: Pool,
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
    #[allow(dead_code)]
    pub(crate) async fn from_path(path: impl Into<PathBuf>) -> Result<Self, Whatever> {
        let pool = Config::new(path)
            .create_pool(Runtime::Tokio1)
            .with_whatever_context(|error| {
                format!("Failed to create Database Connection Pool: {:?}", error)
            })?;
        Self::setup(&pool).await?;
        Ok(Self { pool })
    }

    /// Creates a new handle to the database given an existing connection pool.
    /// On creation, it will:
    /// 1. Create the connection pool
    /// 2. Run the necessary migrations on it
    /// 3. Set up any configuration in the database (such as journal_mode)
    ///
    /// This function is primarily useful for migrating direct usages of the connection
    /// pool over to this new struct. This way, the existing pool can be used side-by-side
    /// with this.
    ///
    /// # Parameters
    /// `pool`: an existing connection pool
    pub(crate) async fn from_pool(pool: Pool) -> Result<Self, Whatever> {
        Self::setup(&pool).await?;
        Ok(Self { pool })
    }

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

    #[tracing::instrument(name = "Querying a single record from the database", skip(row_mapper))]
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
        let pooled_conn = self.pool.get().await.context(DatabasePoolSnafu)?;
        let result = pooled_conn
            .interact(move |conn| {
                conn.prepare_cached(sql)
                    .and_then(move |mut statement| {
                        statement.query_row(parameters, row_mapper).optional()
                    })
            })
            .await
            .context(DatabaseInteractSnafu)?
            .context(DatabaseConnectionSnafu)?;
        Ok(result)
    }
}
