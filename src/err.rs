use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use snafu::prelude::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },

    #[snafu(display("I/O error: {source}"))]
    Io { source: std::io::Error },

    #[snafu(display("Database migration error"))]
    DatabaseMigration {},

    #[snafu(display("Database query error: {source}"))]
    DatabaseQuery { source: rusqlite::Error },

    #[snafu(display("Database data mapping error: {source}"))]
    DatabaseDataMapping { source: serde_rusqlite::Error },

    #[snafu(display("Expected {expected}; got {got} records"))]
    DatabaseMismatchedCount { expected: usize, got: usize },

    #[snafu(display("Environment variables error: {source}"))]
    EnvironmentVariables { source: dotenvy::Error },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self),
        )
            .into_response()
    }
}

impl From<rusqlite::Error> for Error {
    fn from(source: rusqlite::Error) -> Self {
        Error::DatabaseQuery { source }
    }
}

impl From<serde_rusqlite::Error> for Error {
    fn from(source: serde_rusqlite::Error) -> Self {
        Error::DatabaseDataMapping { source }
    }
}

impl From<dotenvy::Error> for Error {
    fn from(source: dotenvy::Error) -> Self {
        Error::EnvironmentVariables { source }
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Error::Io { source }
    }
}
