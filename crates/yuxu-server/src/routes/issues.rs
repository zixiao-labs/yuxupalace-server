use crate::error::AppError;
use axum::{Json, extract::Path};
use raidian::{Issue, ListIssuesResponse};

pub async fn list(Path(_full_name): Path<String>) -> Result<Json<ListIssuesResponse>, AppError> {
    Err(AppError::NotImplemented)
}
pub async fn create(Path(_full_name): Path<String>) -> Result<Json<Issue>, AppError> {
    Err(AppError::NotImplemented)
}
