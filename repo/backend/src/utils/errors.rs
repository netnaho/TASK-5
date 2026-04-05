use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Request;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub status: u16,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(status: Status, message: impl Into<String>) -> Self {
        Self {
            status: status.code,
            error: status.reason_lossy().to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(Status::BadRequest, msg)
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(Status::Unauthorized, msg)
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(Status::Forbidden, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(Status::NotFound, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(Status::InternalServerError, msg)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AppError> for (Status, Json<ApiError>) {
    fn from(err: AppError) -> Self {
        tracing::error!(error = %err, "Application error");
        match err {
            AppError::Database(_) => (
                Status::InternalServerError,
                Json(ApiError::internal("A database error occurred")),
            ),
            AppError::Auth(msg) => (Status::Unauthorized, Json(ApiError::unauthorized(msg))),
            AppError::Validation(msg) => (Status::BadRequest, Json(ApiError::bad_request(msg))),
            AppError::NotFound(msg) => (Status::NotFound, Json(ApiError::not_found(msg))),
            AppError::Forbidden(msg) => (Status::Forbidden, Json(ApiError::forbidden(msg))),
            AppError::Internal(msg) => {
                (Status::InternalServerError, Json(ApiError::internal(msg)))
            }
        }
    }
}

#[catch(403)]
pub fn forbidden(_req: &Request) -> Json<ApiError> {
    Json(ApiError::forbidden("Insufficient permissions"))
}

#[catch(404)]
pub fn not_found(_req: &Request) -> Json<ApiError> {
    Json(ApiError::not_found("Resource not found"))
}

#[catch(401)]
pub fn unauthorized(_req: &Request) -> Json<ApiError> {
    Json(ApiError::unauthorized("Authentication required"))
}

#[catch(422)]
pub fn unprocessable(_req: &Request) -> Json<ApiError> {
    Json(ApiError::bad_request("Invalid request data"))
}

#[catch(429)]
pub fn too_many_requests(_req: &Request) -> Json<ApiError> {
    Json(ApiError::new(Status::TooManyRequests, "Rate limit exceeded. Try again in a minute."))
}

#[catch(500)]
pub fn internal_error(_req: &Request) -> Json<ApiError> {
    Json(ApiError::internal("An internal error occurred"))
}
