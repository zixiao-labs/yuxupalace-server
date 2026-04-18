use crate::error::AppError;
use axum::{Json, extract::Path};
use raidian::{ListMembersResponse, RepositoryMember};

pub async fn list(Path(_full_name): Path<String>) -> Result<Json<ListMembersResponse>, AppError> {
    Ok(Json(ListMembersResponse {
        members: Vec::new(),
    }))
}
pub async fn add(Path(_full_name): Path<String>) -> Result<Json<RepositoryMember>, AppError> {
    Err(AppError::BadRequest("not implemented".into()))
}
