use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize)]
pub struct MrRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author_id: Uuid,
    pub merged_by: Option<Uuid>,
    pub merged_at: Option<DateTime<Utc>>,
    pub ci_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize)]
pub struct MrCommentRow {
    pub id: Uuid,
    pub mr_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
struct CountRow {
    pub count: i64,
}

#[derive(sqlx::FromRow, Debug, Clone)]
struct ReviewerRow {
    pub user_id: Uuid,
}

pub async fn create(
    pool: &PgPool,
    repo_id: Uuid,
    title: &str,
    body: Option<&str>,
    source_branch: &str,
    target_branch: &str,
    author_id: Uuid,
    reviewer_ids: &[Uuid],
    label_ids: &[Uuid],
) -> Result<MrRow> {
    // Retry up to 3 times to handle race conditions in number allocation
    const MAX_RETRIES: u32 = 3;
    let mut retry_count = 0;

    loop {
        let mut tx = pool.begin().await?;

        let result = sqlx::query_as::<_, MrRow>(
            r#"
            INSERT INTO merge_requests (id, repo_id, number, title, body, state, source_branch, target_branch, author_id, created_at, updated_at)
            VALUES (
                $1, $2,
                COALESCE((SELECT MAX(number) FROM merge_requests WHERE repo_id = $2), 0) + 1,
                $3, $4, 'open', $5, $6, $7, NOW(), NOW()
            )
            RETURNING id, repo_id, number, title, body, state, source_branch, target_branch, author_id, merged_by, merged_at, ci_status, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(repo_id)
        .bind(title)
        .bind(body)
        .bind(source_branch)
        .bind(target_branch)
        .bind(author_id)
        .fetch_one(&mut *tx)
        .await;

        let mr = match result {
            Ok(mr) => mr,
            Err(e) => {
                // Check for unique constraint violation (SQLSTATE 23505)
                if let Some(db_err) = e.as_database_error() {
                    if db_err.code().as_deref() == Some("23505") && retry_count < MAX_RETRIES {
                        retry_count += 1;
                        continue;
                    }
                }
                return Err(e.into());
            }
        };

        for reviewer_id in reviewer_ids {
            sqlx::query(
                r#"
                INSERT INTO mr_reviewers (mr_id, user_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(mr.id)
            .bind(reviewer_id)
            .execute(&mut *tx)
            .await?;
        }

        for label_id in label_ids {
            sqlx::query(
                r#"
                INSERT INTO mr_labels (mr_id, label_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(mr.id)
            .bind(label_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        return Ok(mr);
    }
}

pub async fn find_by_number(
    pool: &PgPool,
    repo_id: Uuid,
    number: i32,
) -> Result<Option<MrRow>> {
    let row = sqlx::query_as::<_, MrRow>(
        r#"
        SELECT id, repo_id, number, title, body, state, source_branch, target_branch, author_id, merged_by, merged_at, ci_status, created_at, updated_at
        FROM merge_requests
        WHERE repo_id = $1 AND number = $2
        "#,
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn list(
    pool: &PgPool,
    repo_id: Uuid,
    state_filter: Option<&str>,
    page: i64,
    per_page: i64,
) -> Result<(Vec<MrRow>, i64)> {
    let offset = (page - 1) * per_page;

    let count_row = sqlx::query_as::<_, CountRow>(
        r#"
        SELECT COUNT(*) as count
        FROM merge_requests
        WHERE repo_id = $1
          AND ($2::text IS NULL OR state = $2)
        "#,
    )
    .bind(repo_id)
    .bind(state_filter)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, MrRow>(
        r#"
        SELECT id, repo_id, number, title, body, state, source_branch, target_branch, author_id, merged_by, merged_at, ci_status, created_at, updated_at
        FROM merge_requests
        WHERE repo_id = $1
          AND ($2::text IS NULL OR state = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(repo_id)
    .bind(state_filter)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((rows, count_row.count))
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    body: Option<&str>,
    state: Option<&str>,
) -> Result<MrRow> {
    // Validate that state is not being set to 'merged' - callers should use set_merged() instead
    if state == Some("merged") {
        return Err(anyhow::anyhow!(
            "cannot set state to 'merged' via update(); use set_merged() instead"
        ));
    }

    let row = sqlx::query_as::<_, MrRow>(
        r#"
        UPDATE merge_requests
        SET title = COALESCE($2, title),
            body = COALESCE($3, body),
            state = COALESCE($4, state),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, repo_id, number, title, body, state, source_branch, target_branch, author_id, merged_by, merged_at, ci_status, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(body)
    .bind(state)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn set_merged(pool: &PgPool, id: Uuid, merged_by: Uuid) -> Result<MrRow> {
    let row = sqlx::query_as::<_, MrRow>(
        r#"
        UPDATE merge_requests
        SET state = 'merged',
            merged_by = $2,
            merged_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, repo_id, number, title, body, state, source_branch, target_branch, author_id, merged_by, merged_at, ci_status, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(merged_by)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update_ci_status(pool: &PgPool, id: Uuid, status: &str) -> Result<()> {
    let result = sqlx::query(
        r#"
        UPDATE merge_requests
        SET ci_status = $2, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("merge request not found"));
    }

    Ok(())
}

pub async fn add_comment(
    pool: &PgPool,
    mr_id: Uuid,
    author_id: Uuid,
    body: &str,
    file_path: Option<&str>,
    line_number: Option<i32>,
) -> Result<MrCommentRow> {
    let row = sqlx::query_as::<_, MrCommentRow>(
        r#"
        INSERT INTO mr_comments (id, mr_id, author_id, body, file_path, line_number, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
        RETURNING id, mr_id, author_id, body, file_path, line_number, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(mr_id)
    .bind(author_id)
    .bind(body)
    .bind(file_path)
    .bind(line_number)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_comments(pool: &PgPool, mr_id: Uuid) -> Result<Vec<MrCommentRow>> {
    let rows = sqlx::query_as::<_, MrCommentRow>(
        r#"
        SELECT id, mr_id, author_id, body, file_path, line_number, created_at, updated_at
        FROM mr_comments
        WHERE mr_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(mr_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn list_reviewers(pool: &PgPool, mr_id: Uuid) -> Result<Vec<Uuid>> {
    let rows = sqlx::query_as::<_, ReviewerRow>(
        r#"
        SELECT user_id
        FROM mr_reviewers
        WHERE mr_id = $1
        "#,
    )
    .bind(mr_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.user_id).collect())
}