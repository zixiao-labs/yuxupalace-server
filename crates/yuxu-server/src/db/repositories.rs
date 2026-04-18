use super::DbPool;
use crate::error::AppError;
use sqlx::Row;

#[derive(Debug, Clone)]
pub struct RepoRecord {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub is_private: bool,
    pub default_branch: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub async fn insert(pool: &DbPool, r: &RepoRecord) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO repositories (id, owner_id, name, full_name, description, is_private, default_branch, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(&r.id)
    .bind(&r.owner_id)
    .bind(&r.name)
    .bind(&r.full_name)
    .bind(&r.description)
    .bind(r.is_private)
    .bind(&r.default_branch)
    .bind(r.created_at)
    .bind(r.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_by_owner(pool: &DbPool, owner_id: &str) -> Result<Vec<RepoRecord>, AppError> {
    let rows = sqlx::query(
        "SELECT id, owner_id, name, full_name, description, is_private, default_branch, created_at, updated_at FROM repositories WHERE owner_id = $1 ORDER BY created_at DESC",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;
    rows.iter().map(from_row).collect()
}

pub async fn find_by_full_name(
    pool: &DbPool,
    full_name: &str,
) -> Result<Option<RepoRecord>, AppError> {
    let row = sqlx::query(
        "SELECT id, owner_id, name, full_name, description, is_private, default_branch, created_at, updated_at FROM repositories WHERE full_name = $1",
    )
    .bind(full_name)
    .fetch_optional(pool)
    .await?;
    match row {
        Some(r) => Ok(Some(from_row(&r)?)),
        None => Ok(None),
    }
}

#[cfg(feature = "postgres")]
fn from_row(row: &sqlx::postgres::PgRow) -> Result<RepoRecord, AppError> {
    Ok(RepoRecord {
        id: row.try_get("id")?,
        owner_id: row.try_get("owner_id")?,
        name: row.try_get("name")?,
        full_name: row.try_get("full_name")?,
        description: row.try_get("description")?,
        is_private: row.try_get("is_private")?,
        default_branch: row.try_get("default_branch")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<RepoRecord, AppError> {
    Ok(RepoRecord {
        id: row.try_get("id")?,
        owner_id: row.try_get("owner_id")?,
        name: row.try_get("name")?,
        full_name: row.try_get("full_name")?,
        description: row.try_get("description")?,
        is_private: row.try_get::<i64, _>("is_private")? != 0,
        default_branch: row.try_get("default_branch")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
