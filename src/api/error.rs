use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::dto::common::ErrorResponse;
use crate::errors::{AppError, CodedKind};

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::Configuration(_) => (StatusCode::SERVICE_UNAVAILABLE, "configuration_error"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            AppError::TooManyRequests(_) => (StatusCode::TOO_MANY_REQUESTS, "too_many_requests"),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_error"),
            AppError::Persistence(_) | AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
            AppError::Coded { kind, code, .. } => {
                let status = match kind {
                    CodedKind::Conflict => StatusCode::CONFLICT,
                    CodedKind::Forbidden => StatusCode::FORBIDDEN,
                    CodedKind::NotFound => StatusCode::NOT_FOUND,
                    CodedKind::Validation => StatusCode::UNPROCESSABLE_ENTITY,
                };
                (status, *code)
            }
        };
        let message = match &self {
            AppError::Persistence(_) | AppError::Internal(_) => {
                tracing::error!(error = %self, "request failed with an internal error");
                "internal server error".to_owned()
            }
            _ => self.client_message().to_owned(),
        };
        (
            status,
            Json(ErrorResponse {
                code: code.to_owned(),
                message,
            }),
        )
            .into_response()
    }
}
