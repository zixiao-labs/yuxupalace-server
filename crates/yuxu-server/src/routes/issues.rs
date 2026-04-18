use crate::error::AppError;
use axum::{Json, extract::Path};
use raidian::{Issue, ListIssuesResponse};

pub async fn list(Path(_full_name): Path<String>) -> Result<Json<ListIssuesResponse>, AppError> {
    Ok(Json(ListIssuesResponse {
        issues: Vec::new(),
        total: 0,
    }))
}
pub async fn create(Path(_full_name): Path<String>) -> Result<Json<Issue>, AppError> {
    Err(AppError::BadRequest("not implemented".into()))
}
