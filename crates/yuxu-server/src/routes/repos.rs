use crate::{app_state::AppState, db, error::AppError, middleware::auth::AuthUser};
use axum::{
    Json,
    extract::{Path, State},
};
use raidian::{CreateRepositoryRequest, Repository};

fn to_pb(r: &db::repositories::RepoRecord, owner_username: &str) -> Repository {
    Repository {
        id: r.id.clone(),
        owner_id: r.owner_id.clone(),
        owner_username: owner_username.to_string(),
        name: r.name.clone(),
        full_name: r.full_name.clone(),
        description: r.description.clone(),
        is_private: r.is_private,
        default_branch: r.default_branch.clone(),
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

pub async fn list(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<Repository>>, AppError> {
    let repos = db::repositories::list_by_owner(&state.db, &claims.sub).await?;
    Ok(Json(
        repos.iter().map(|r| to_pb(r, &claims.username)).collect(),
    ))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateRepositoryRequest>,
) -> Result<Json<Repository>, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("name required".into()));
    }
    let full_name = format!("{}/{}", claims.username, req.name);
    if db::repositories::find_by_full_name(&state.db, &full_name)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("repository already exists".into()));
    }
    let now = chrono::Utc::now().timestamp();
    let rec = db::repositories::RepoRecord {
        id: uuid::Uuid::new_v4().to_string(),
        owner_id: claims.sub.clone(),
        name: req.name,
        full_name,
        description: req.description,
        is_private: req.is_private,
        default_branch: "main".into(),
        created_at: now,
        updated_at: now,
    };
    db::repositories::insert(&state.db, &rec).await?;
    Ok(Json(to_pb(&rec, &claims.username)))
}

pub async fn get_by_name(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(full_name): Path<String>,
) -> Result<Json<Repository>, AppError> {
    let rec = db::repositories::find_by_full_name(&state.db, &full_name)
        .await?
        .ok_or_else(|| AppError::NotFound("repository".into()))?;
    Ok(Json(to_pb(&rec, &claims.username)))
}
