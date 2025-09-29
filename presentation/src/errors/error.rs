use application::AppError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub struct ApiError(AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            AppError::BadRequest(reason) => (StatusCode::BAD_REQUEST, reason),
            AppError::Unauthorized() => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden() => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::ServerBusy() => (StatusCode::SERVICE_UNAVAILABLE, "Server busy".to_string()),
            AppError::Inconsistent(reason) => (StatusCode::CONFLICT, reason),
            AppError::AccountLocked() => (StatusCode::UNAUTHORIZED, "Account locked".to_string()),
            AppError::Unexpected(ex) => {
                tracing::error!("Internal server error: {:?}", ex);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred".to_string(),
                )
            }
            AppError::DataNotFound(reason) => (StatusCode::NOT_FOUND, reason),
            AppError::DataConflict(reason) => (StatusCode::CONFLICT, reason),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        Self(error)
    }
}
