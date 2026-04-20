use crate::app_state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub mod auth;
pub mod dashboard;
pub mod issues;
pub mod members;
pub mod merge_requests;
pub mod pipelines;
pub mod repos;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/config", get(auth::config))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/github/callback", post(auth::github_callback))
        .route("/api/auth/zixiao/callback", post(auth::zixiao_callback))
        .route("/api/repos", get(repos::list).post(repos::create))
        .route("/api/repos/{full_name}", get(repos::get_by_name))
        .route(
            "/api/repos/{full_name}/issues",
            get(issues::list).post(issues::create),
        )
        .route(
            "/api/repos/{full_name}/merge_requests",
            get(merge_requests::list).post(merge_requests::create),
        )
        .route(
            "/api/repos/{full_name}/pipelines",
            get(pipelines::list).post(pipelines::trigger),
        )
        .route(
            "/api/repos/{full_name}/members",
            get(members::list).post(members::add),
        )
        .route("/api/dashboard", get(dashboard::stats))
}
