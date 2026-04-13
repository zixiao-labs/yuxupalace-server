use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("database error: {0}")]
    Database(String),

    #[error("git error: {0}")]
    Git(String),
}

impl From<git2::Error> for AppError {
    fn from(err: git2::Error) -> Self {
        AppError::Git(err.message().to_string())
    }
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Unauthorized => 401,
            AppError::Forbidden(_) => 403,
            AppError::NotFound(_) => 404,
            AppError::Conflict(_) => 409,
            AppError::Validation(_) => 422,
            AppError::Internal(_) => 500,
            AppError::Database(_) => 500,
            AppError::Git(_) => 500,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Git(_) => "GIT_ERROR",
        }
    }
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl From<&AppError> for ErrorResponse {
    fn from(err: &AppError) -> Self {
        let error = match err {
            // For internal/server errors, return a generic message to avoid leaking details
            AppError::Internal(_) | AppError::Database(_) | AppError::Git(_) => {
                "Internal server error".to_string()
            }
            // For client errors, return the actual error message
            _ => err.to_string(),
        };
        ErrorResponse {
            error,
            code: err.error_code().to_string(),
        }
    }
}