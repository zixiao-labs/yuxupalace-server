use crate::error::AppError;
use axum::Json;
use raidian::DashboardStats;

pub async fn stats() -> Result<Json<DashboardStats>, AppError> {
    Err(AppError::NotImplemented)
}
