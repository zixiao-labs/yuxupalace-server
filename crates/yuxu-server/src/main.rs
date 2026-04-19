mod app_state;
mod collab;
mod config;
mod db;
mod error;
mod middleware;
mod routes;

use app_state::AppState;
use axum::{Router, http::HeaderValue, routing::get};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
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

    let cors = build_cors_layer(cfg.cors_allowed_origins.as_deref());

    let app = Router::new()
        .merge(routes::router())
        .route("/rpc", get(collab::ws::handler))
        .route("/health", get(|| async { "ok" }))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    tracing::info!(bind = %cfg.bind, "yuxu-server listening");
    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Build a CORS layer from `YUXU_CORS_ORIGINS`. Comma-separated list; `*`
/// allows any origin (suitable for local dev only). An empty/unset value
/// disables CORS entirely so production deployments behind a reverse proxy
/// don't advertise cross-origin access by accident.
fn build_cors_layer(raw: Option<&str>) -> CorsLayer {
    use axum::http::Method;
    use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
    let base = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
        .allow_credentials(false);
    match raw {
        None | Some("") => base,
        Some("*") => base.allow_origin(AllowOrigin::any()),
        Some(list) => {
            let origins: Vec<HeaderValue> = list
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<HeaderValue>().ok())
                .collect();
            base.allow_origin(origins)
        }
    }
}
