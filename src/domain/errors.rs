use actix_web::{self, HttpResponse, ResponseError};
use argon2::password_hash;
use deadpool_sqlite::{rusqlite, InteractError, PoolError};
use snafu::{prelude::*, Backtrace};
use tokio::task::JoinError;

#[derive(Debug, Snafu)]
pub(crate) struct ServerError(pub(crate) InnerError);

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        // TODO: put more information here, such as some sort of Trace ID
        //       so I can look in the logs using that Trace ID later.
        HttpResponse::new(self.status_code())
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum InnerError {
    #[snafu(display("Failed to get a connection from the connection pool"))]
    DatabasePoolError {
        source: PoolError,
        backtrace: Backtrace,
    },
    #[snafu(display("Failed while connecting to/running queries on the database"))]
    DatabaseConnectionError {
        source: rusqlite::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Failed while running interact"))]
    DatabaseInteractError {
        source: InteractError,
        backtrace: Backtrace,
    },
    PasswordHashingError {
        error_source: password_hash::errors::Error,
    },
    #[snafu(display("Failed to join a thread/task with the current one"))]
    JoinError {
        source: JoinError,
        backtrace: Backtrace,
    },
    InvalidUsernameOrPassword
}
