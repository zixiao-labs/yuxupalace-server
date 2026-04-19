use super::DbPool;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: String,
    pub bio: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub github_id: Option<String>,
}

pub async fn insert(pool: &DbPool, u: &UserRecord) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO users (id, username, email, display_name, avatar_url, bio, password_hash, is_admin, created_at, updated_at, github_id) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
    )
    .bind(&u.id)
    .bind(&u.username)
    .bind(&u.email)
    .bind(&u.display_name)
    .bind(&u.avatar_url)
    .bind(&u.bio)
    .bind(&u.password_hash)
    .bind(u.is_admin)
    .bind(u.created_at)
    .bind(u.updated_at)
    .bind(u.github_id.as_deref())
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn find_by_username_or_email(
    pool: &DbPool,
    ident: &str,
) -> Result<Option<UserRecord>, AppError> {
    let row = sqlx::query(
        "SELECT id, username, email, display_name, avatar_url, bio, password_hash, is_admin, created_at, updated_at, github_id FROM users WHERE username = $1 OR email = $1",
    )
    .bind(ident)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;
    match row {
        Some(r) => Ok(Some(user_from_row(&r)?)),
        None => Ok(None),
    }
}

pub async fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<UserRecord>, AppError> {
    let row = sqlx::query(
        "SELECT id, username, email, display_name, avatar_url, bio, password_hash, is_admin, created_at, updated_at, github_id FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;
    match row {
        Some(r) => Ok(Some(user_from_row(&r)?)),
        None => Ok(None),
    }
}

pub async fn find_by_github_id(
    pool: &DbPool,
    github_id: &str,
) -> Result<Option<UserRecord>, AppError> {
    let row = sqlx::query(
        "SELECT id, username, email, display_name, avatar_url, bio, password_hash, is_admin, created_at, updated_at, github_id FROM users WHERE github_id = $1",
    )
    .bind(github_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;
    match row {
        Some(r) => Ok(Some(user_from_row(&r)?)),
        None => Ok(None),
    }
}

pub async fn link_github_id(pool: &DbPool, user_id: &str, github_id: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET github_id = $1, updated_at = $2 WHERE id = $3")
        .bind(github_id)
        .bind(chrono::Utc::now().timestamp())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
    Ok(())
}

#[cfg(feature = "postgres")]
fn user_from_row(row: &sqlx::postgres::PgRow) -> Result<UserRecord, AppError> {
    Ok(UserRecord {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        avatar_url: row.try_get("avatar_url")?,
        bio: row.try_get("bio")?,
        password_hash: row.try_get("password_hash")?,
        is_admin: row.try_get("is_admin")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        github_id: row.try_get("github_id")?,
    })
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
fn user_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<UserRecord, AppError> {
    Ok(UserRecord {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        avatar_url: row.try_get("avatar_url")?,
        bio: row.try_get("bio")?,
        password_hash: row.try_get("password_hash")?,
        is_admin: row.try_get::<i64, _>("is_admin")? != 0,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        github_id: row.try_get("github_id")?,
    })
}
