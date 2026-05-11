use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub error: String,
    pub kind: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StoreError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not ready: {0}")]
    NotReady(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl StoreError {
    pub fn kind(&self) -> &'static str {
        match self {
            StoreError::BadRequest(_) => "bad_request",
            StoreError::NotReady(_) => "not_ready",
            StoreError::Internal(_) => "internal",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            StoreError::BadRequest(_) => StatusCode::BAD_REQUEST,
            StoreError::NotReady(_) => StatusCode::SERVICE_UNAVAILABLE,
            StoreError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for StoreError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorResponse {
            error: self.to_string(),
            kind: self.kind().to_string(),
        };
        (status, Json(body)).into_response()
    }
}
