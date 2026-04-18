use crate::error::AppError;
use axum::{Json, extract::Path};
use raidian::{ListMergeRequestsResponse, MergeRequest};

pub async fn list(
    Path(_full_name): Path<String>,
) -> Result<Json<ListMergeRequestsResponse>, AppError> {
    Err(AppError::NotImplemented)
}
pub async fn create(Path(_full_name): Path<String>) -> Result<Json<MergeRequest>, AppError> {
    Err(AppError::NotImplemented)
}
