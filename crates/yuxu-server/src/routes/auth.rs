use crate::{app_state::AppState, db, error::AppError, middleware::auth::AuthUser};
use axum::{Json, extract::State};
use raidian::{AuthResponse, LoginRequest, RegisterRequest, UserProfile};
use yuxu_core::auth::{hash_password, verify_password};

fn profile_from_record(u: &db::users::UserRecord) -> UserProfile {
    UserProfile {
        id: u.id.clone(),
        username: u.username.clone(),
        email: u.email.clone(),
        display_name: u.display_name.clone(),
        avatar_url: u.avatar_url.clone(),
        bio: u.bio.clone(),
        is_admin: u.is_admin,
        created_at: u.created_at,
        updated_at: u.updated_at,
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if req.username.trim().is_empty() || req.password.len() < 8 {
        return Err(AppError::BadRequest("invalid credentials".into()));
    }
    if db::users::find_by_username_or_email(&state.db, &req.username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("username already taken".into()));
    }
    let now = chrono::Utc::now().timestamp();
    let rec = db::users::UserRecord {
        id: uuid::Uuid::new_v4().to_string(),
        username: req.username.clone(),
        email: req.email.clone(),
        display_name: req.display_name,
        avatar_url: String::new(),
        bio: String::new(),
        password_hash: hash_password(&req.password)?,
        is_admin: false,
        created_at: now,
        updated_at: now,
    };
    db::users::insert(&state.db, &rec).await?;
    let token = state.jwt.issue(&rec.id, &rec.username, rec.is_admin)?;
    Ok(Json(AuthResponse {
        token,
        user: Some(profile_from_record(&rec)),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = db::users::find_by_username_or_email(&state.db, &req.username_or_email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;
    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }
    let token = state.jwt.issue(&user.id, &user.username, user.is_admin)?;
    Ok(Json(AuthResponse {
        token,
        user: Some(profile_from_record(&user)),
    }))
}

pub async fn me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<UserProfile>, AppError> {
    let user = db::users::find_by_id(&state.db, &claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound("user".into()))?;
    Ok(Json(profile_from_record(&user)))
}
