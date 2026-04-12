use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize)]
pub struct RepoRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub org_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub default_branch: String,
    #[serde(skip_serializing)]
    pub disk_path: String,
    pub stars_count: i32,
    pub forks_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
struct CountRow {
    pub count: i64,
}

pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    name: &str,
    description: Option<&str>,
    is_private: bool,
    default_branch: &str,
    disk_path: &str,
) -> Result<RepoRow> {
    let row = sqlx::query_as::<_, RepoRow>(
        r#"
        INSERT INTO repositories (id, owner_id, name, description, is_private, default_branch, disk_path, stars_count, forks_count, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0, NOW(), NOW())
        RETURNING id, owner_id, org_id, name, description, is_private, default_branch, disk_path, stars_count, forks_count, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(owner_id)
    .bind(name)
    .bind(description)
    .bind(is_private)
    .bind(default_branch)
    .bind(disk_path)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_owner_and_name(
    pool: &PgPool,
    owner_id: Uuid,
    name: &str,
) -> Result<Option<RepoRow>> {
    let row = sqlx::query_as::<_, RepoRow>(
        r#"
        SELECT id, owner_id, org_id, name, description, is_private, default_branch, disk_path, stars_count, forks_count, created_at, updated_at
        FROM repositories
        WHERE owner_id = $1 AND name = $2
        "#,
    )
    .bind(owner_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RepoRow>> {
    let row = sqlx::query_as::<_, RepoRow>(
        r#"
        SELECT id, owner_id, org_id, name, description, is_private, default_branch, disk_path, stars_count, forks_count, created_at, updated_at
        FROM repositories
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn list_by_owner(
    pool: &PgPool,
    owner_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<(Vec<RepoRow>, i64)> {
    let offset = (page - 1) * per_page;

    let count_row = sqlx::query_as::<_, CountRow>(
        r#"
        SELECT COUNT(*) as count
        FROM repositories
        WHERE owner_id = $1
        "#,
    )
    .bind(owner_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, RepoRow>(
        r#"
        SELECT id, owner_id, org_id, name, description, is_private, default_branch, disk_path, stars_count, forks_count, created_at, updated_at
        FROM repositories
        WHERE owner_id = $1
        ORDER BY updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(owner_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((rows, count_row.count))
}

pub async fn list_accessible(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<(Vec<RepoRow>, i64)> {
    let offset = (page - 1) * per_page;

    let count_row = sqlx::query_as::<_, CountRow>(
        r#"
        SELECT COUNT(*) as count
        FROM repositories r
        WHERE r.owner_id = $1
           OR r.id IN (SELECT repo_id FROM members WHERE user_id = $1)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, RepoRow>(
        r#"
        SELECT r.id, r.owner_id, r.org_id, r.name, r.description, r.is_private, r.default_branch, r.disk_path, r.stars_count, r.forks_count, r.created_at, r.updated_at
        FROM repositories r
        WHERE r.owner_id = $1
           OR r.id IN (SELECT repo_id FROM members WHERE user_id = $1)
        ORDER BY r.updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((rows, count_row.count))
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM repositories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}