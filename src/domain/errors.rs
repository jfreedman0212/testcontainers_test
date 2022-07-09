use actix_web::{self, HttpResponse, ResponseError};
use deadpool_sqlite::{rusqlite, InteractError, PoolError};
use snafu::{prelude::*, Backtrace};

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
}
