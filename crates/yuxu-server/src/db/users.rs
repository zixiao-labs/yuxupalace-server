use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
    display_name: &str,
) -> Result<UserRow> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (id, username, email, password_hash, display_name, is_admin, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, false, NOW(), NOW())
        RETURNING id, username, email, display_name, password_hash, avatar_url, bio, is_admin, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, email, display_name, password_hash, avatar_url, bio, is_admin, created_at, updated_at
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, email, display_name, password_hash, avatar_url, bio, is_admin, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_username_or_email(
    pool: &PgPool,
    identifier: &str,
) -> Result<Option<UserRow>> {
    // Detect if identifier looks like an email (contains '@')
    let row = if identifier.contains('@') {
        // Query by email only
        sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, display_name, password_hash, avatar_url, bio, is_admin, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(identifier)
        .fetch_optional(pool)
        .await?
    } else {
        // Query by username only
        sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, display_name, password_hash, avatar_url, bio, is_admin, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(identifier)
        .fetch_optional(pool)
        .await?
    };

    Ok(row)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, email, display_name, password_hash, avatar_url, bio, is_admin, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn update_profile(
    pool: &PgPool,
    id: Uuid,
    display_name: Option<&str>,
    bio: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<UserRow> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users
        SET display_name = COALESCE($2, display_name),
            bio = COALESCE($3, bio),
            avatar_url = COALESCE($4, avatar_url),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, username, email, display_name, password_hash, avatar_url, bio, is_admin, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(display_name)
    .bind(bio)
    .bind(avatar_url)
    .fetch_one(pool)
    .await?;

    Ok(row)
}