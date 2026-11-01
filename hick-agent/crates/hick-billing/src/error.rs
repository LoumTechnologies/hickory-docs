//! Axum error type that serialises into a JSON response.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct AppError(anyhow::Error, StatusCode);

impl AppError {
    pub fn bad_request(msg: impl std::fmt::Display) -> Self {
        Self(anyhow::anyhow!("{}", msg), StatusCode::BAD_REQUEST)
    }
    pub fn unauthorized(msg: impl std::fmt::Display) -> Self {
        Self(anyhow::anyhow!("{}", msg), StatusCode::UNAUTHORIZED)
    }
    pub fn internal(e: impl Into<anyhow::Error>) -> Self {
        Self(e.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        Self(e.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.0.to_string() });
        (self.1, axum::Json(body)).into_response()
    }
}
