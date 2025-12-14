use std::sync::Arc;

use axum::{
    BoxError, Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type Result<T, E = AppError> = std::result::Result<T, E>;

/// JSON error response structure.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// The central error type used for HTTP responses.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("bad request")]
    BadRequest(Option<&'static str>),

    #[error("internal error")]
    Internal(
        #[source]
        #[from]
        eyre::Report,
    ),

    #[error("internal error")]
    InternalBoxed(#[source] BoxError),

    /// Database error
    #[error("database error")]
    Database(
        #[source]
        #[from]
        sqlx::Error,
    ),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Internal(..) | AppError::InternalBoxed(..) | AppError::Database(..) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found"),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.unwrap_or("bad request")),
        };

        let mut response = (
            status,
            Json(ErrorResponse {
                error: message.to_string(),
            }),
        )
            .into_response();

        response.extensions_mut().insert(Arc::new(self));

        response
    }
}
