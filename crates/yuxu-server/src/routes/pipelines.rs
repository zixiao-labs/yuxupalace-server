use crate::error::AppError;
use axum::{Json, extract::Path};
use raidian::pipeline::{ListPipelinesResponse, Pipeline};

pub async fn list(Path(_full_name): Path<String>) -> Result<Json<ListPipelinesResponse>, AppError> {
    Ok(Json(ListPipelinesResponse {
        pipelines: Vec::new(),
        total: 0,
    }))
}
pub async fn trigger(Path(_full_name): Path<String>) -> Result<Json<Pipeline>, AppError> {
    Err(AppError::BadRequest("not implemented".into()))
}
