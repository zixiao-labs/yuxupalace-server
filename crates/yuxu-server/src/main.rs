mod app_state;
mod collab;
mod config;
mod db;
mod error;
mod middleware;
mod routes;

use app_state::AppState;
use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use yuxu_core::auth::JwtService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "yuxu_server=info,tower_http=info".into()),
        )
        .init();

    let cfg = Arc::new(config::Config::from_env()?);
    let db = db::connect(&cfg.database_url).await?;
    db::run_migrations(&db).await?;

    let state = AppState {
        config: cfg.clone(),
        db,
        jwt: Arc::new(JwtService::new(
            cfg.jwt_secret.as_bytes(),
            cfg.jwt_ttl_seconds,
        )),
        hub: Arc::new(collab::CollabHub::new()),
    };

    let app = Router::new()
        .merge(routes::router())
        .route("/rpc", get(collab::ws::handler))
        .route("/health", get(|| async { "ok" }))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    tracing::info!(bind = %cfg.bind, "yuxu-server listening");
    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
