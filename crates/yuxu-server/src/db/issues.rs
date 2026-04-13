use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize)]
pub struct IssueRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize)]
pub struct CommentRow {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
struct CountRow {
    pub count: i64,
}

pub async fn create(
    pool: &PgPool,
    repo_id: Uuid,
    title: &str,
    body: Option<&str>,
    author_id: Uuid,
    assignee_id: Option<Uuid>,
    label_ids: &[Uuid],
) -> Result<IssueRow> {
    // Retry up to 3 times to handle race conditions in number allocation
    const MAX_RETRIES: u32 = 3;
    let mut retry_count = 0;

    loop {
        let mut tx = pool.begin().await?;

        let result = sqlx::query_as::<_, IssueRow>(
            r#"
            INSERT INTO issues (id, repo_id, number, title, body, state, author_id, assignee_id, created_at, updated_at)
            VALUES (
                $1, $2,
                COALESCE((SELECT MAX(number) FROM issues WHERE repo_id = $2), 0) + 1,
                $3, $4, 'open', $5, $6, NOW(), NOW()
            )
            RETURNING id, repo_id, number, title, body, state, author_id, assignee_id, created_at, updated_at, closed_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(repo_id)
        .bind(title)
        .bind(body)
        .bind(author_id)
        .bind(assignee_id)
        .fetch_one(&mut *tx)
        .await;

        let issue = match result {
            Ok(issue) => issue,
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

        for label_id in label_ids {
            sqlx::query(
                r#"
                INSERT INTO issue_labels (issue_id, label_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(issue.id)
            .bind(label_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        return Ok(issue);
    }
}

pub async fn find_by_number(
    pool: &PgPool,
    repo_id: Uuid,
    number: i32,
) -> Result<Option<IssueRow>> {
    let row = sqlx::query_as::<_, IssueRow>(
        r#"
        SELECT id, repo_id, number, title, body, state, author_id, assignee_id, created_at, updated_at, closed_at
        FROM issues
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
) -> Result<(Vec<IssueRow>, i64)> {
    // Validate pagination inputs
    if page <= 0 {
        return Err(anyhow::anyhow!("invalid pagination: page must be greater than 0"));
    }
    if per_page <= 0 {
        return Err(anyhow::anyhow!("invalid pagination: per_page must be greater than 0"));
    }

    let offset = (page - 1) * per_page;

    let count_row = sqlx::query_as::<_, CountRow>(
        r#"
        SELECT COUNT(*) as count
        FROM issues
        WHERE repo_id = $1
          AND ($2::text IS NULL OR state = $2)
        "#,
    )
    .bind(repo_id)
    .bind(state_filter)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, IssueRow>(
        r#"
        SELECT id, repo_id, number, title, body, state, author_id, assignee_id, created_at, updated_at, closed_at
        FROM issues
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
    assignee_id: Option<Option<Uuid>>,
) -> Result<IssueRow> {
    // Validate state if provided
    if let Some(s) = state {
        if s != "open" && s != "closed" {
            return Err(anyhow::anyhow!(
                "invalid state '{}': must be 'open' or 'closed'",
                s
            ));
        }
    }

    let (set_assignee, assignee_value) = match assignee_id {
        Some(inner) => (true, inner),
        None => (false, None),
    };

    let row = sqlx::query_as::<_, IssueRow>(
        r#"
        UPDATE issues
        SET title = COALESCE($2, title),
            body = COALESCE($3, body),
            state = COALESCE($4, state),
            assignee_id = CASE WHEN $6 THEN $5 ELSE assignee_id END,
            closed_at = CASE
                WHEN $4 = 'closed' AND closed_at IS NULL THEN NOW()
                WHEN $4 = 'open' THEN NULL
                ELSE closed_at
            END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, repo_id, number, title, body, state, author_id, assignee_id, created_at, updated_at, closed_at
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(body)
    .bind(state)
    .bind(assignee_value)
    .bind(set_assignee)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn add_comment(
    pool: &PgPool,
    issue_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> Result<CommentRow> {
    let row = sqlx::query_as::<_, CommentRow>(
        r#"
        INSERT INTO issue_comments (id, issue_id, author_id, body, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        RETURNING id, issue_id, author_id, body, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(issue_id)
    .bind(author_id)
    .bind(body)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_comments(pool: &PgPool, issue_id: Uuid) -> Result<Vec<CommentRow>> {
    let rows = sqlx::query_as::<_, CommentRow>(
        r#"
        SELECT id, issue_id, author_id, body, created_at, updated_at
        FROM issue_comments
        WHERE issue_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn count_comments(pool: &PgPool, issue_id: Uuid) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>(
        r#"
        SELECT COUNT(*) as count
        FROM issue_comments
        WHERE issue_id = $1
        "#,
    )
    .bind(issue_id)
    .fetch_one(pool)
    .await?;

    Ok(row.count)
}