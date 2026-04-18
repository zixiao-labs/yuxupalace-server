//! Feature-gated database pool. `postgres` and `sqlite` features pick the driver.

use anyhow::Result;

#[cfg(feature = "postgres")]
pub type DbPool = sqlx::PgPool;
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub type DbPool = sqlx::SqlitePool;

#[cfg(feature = "postgres")]
pub async fn connect(url: &str) -> Result<DbPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(16)
        .connect(url)
        .await?;
    Ok(pool)
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub async fn connect(url: &str) -> Result<DbPool> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;
    let opts = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(16)
        .connect_with(opts)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    #[cfg(feature = "postgres")]
    sqlx::migrate!("./migrations/postgres").run(pool).await?;
    #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
    sqlx::migrate!("./migrations/sqlite").run(pool).await?;
    Ok(())
}

pub mod repositories;
pub mod users;
