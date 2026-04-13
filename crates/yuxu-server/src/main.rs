mod app_state;
mod config;
pub mod db;
pub mod middleware;
pub mod routes;
pub mod ws;

use app_state::AppState;
use config::Config;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!("Starting YuXu DevOps server");

    // Ensure git root directory exists
    std::fs::create_dir_all(&config.git_root)?;
    tracing::info!("Git root: {}", config.git_root.display());

    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;
    tracing::info!("Connected to database");

    // Run migrations
    sqlx::migrate!("../../migrations").run(&pool).await?;
    tracing::info!("Database migrations applied");

    let state = AppState::new(pool, config.git_root, config.jwt_secret);

    let app = routes::api_router(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr()).await?;
    tracing::info!("Listening on {}", config.listen_addr());

    axum::serve(listener, app).await?;

    Ok(())
}