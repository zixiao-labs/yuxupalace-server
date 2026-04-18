use crate::error::AppError;
use axum::Json;
use raidian::DashboardStats;

pub async fn stats() -> Result<Json<DashboardStats>, AppError> {
    Ok(Json(DashboardStats {
        repo_count: 0,
        open_issues: 0,
        open_merge_requests: 0,
        pipeline_pass_rate: 0.0,
        recent_activities: Vec::new(),
    }))
}
