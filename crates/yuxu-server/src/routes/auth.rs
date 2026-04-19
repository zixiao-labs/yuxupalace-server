use crate::{app_state::AppState, db, error::AppError, middleware::auth::AuthUser};
use axum::{Json, extract::State};
use raidian::{AuthResponse, GithubOauthRequest, LoginRequest, RegisterRequest, UserProfile};
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
    // `find_by_username_or_email` matches the single argument against either
    // column, so a username collision *and* an email collision have to be
    // checked independently.
    let username_taken = db::users::find_by_username_or_email(&state.db, &req.username)
        .await?
        .is_some();
    let email_taken = !req.email.is_empty()
        && db::users::find_by_username_or_email(&state.db, &req.email)
            .await?
            .is_some();
    if username_taken || email_taken {
        return Err(AppError::Conflict("username or email already taken".into()));
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
        github_id: None,
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

#[derive(serde::Deserialize)]
struct GithubAccessTokenResp {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(serde::Deserialize)]
struct GithubUser {
    id: u64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
    email: Option<String>,
    bio: Option<String>,
}

#[derive(serde::Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

pub async fn github_callback(
    State(state): State<AppState>,
    Json(req): Json<GithubOauthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let client_id = state
        .config
        .github_client_id
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("github oauth not configured".into()))?;
    let client_secret = state
        .config
        .github_client_secret
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("github oauth not configured".into()))?;
    if req.code.trim().is_empty() {
        return Err(AppError::BadRequest("missing code".into()));
    }
    // `state` is validated by the frontend against sessionStorage. The server
    // receives it for defense-in-depth logging but does not store per-flow
    // state — by the time the code reaches us, GitHub has already bound the
    // code to our client_id.

    let http = reqwest::Client::builder()
        .user_agent("yuxu-server/github-oauth")
        .build()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("build http client: {e}")))?;

    let token_resp: GithubAccessTokenResp = http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", req.code.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("github token exchange: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("decode github token: {e}")))?;

    if let Some(err) = token_resp.error {
        let desc = token_resp.error_description.unwrap_or_default();
        tracing::warn!(%err, %desc, "github returned oauth error");
        return Err(AppError::Unauthorized("github oauth failed".into()));
    }
    let access_token = token_resp
        .access_token
        .ok_or_else(|| AppError::Unauthorized("github oauth failed".into()))?;

    let gh_user: GithubUser = http
        .get("https://api.github.com/user")
        .bearer_auth(&access_token)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("github /user: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("decode github user: {e}")))?;

    let email = match gh_user.email.clone() {
        Some(e) if !e.is_empty() => e,
        _ => {
            let emails: Vec<GithubEmail> = http
                .get("https://api.github.com/user/emails")
                .bearer_auth(&access_token)
                .header("Accept", "application/vnd.github+json")
                .send()
                .await
                .map_err(|e| AppError::Anyhow(anyhow::anyhow!("github /user/emails: {e}")))?
                .json()
                .await
                .map_err(|e| AppError::Anyhow(anyhow::anyhow!("decode github emails: {e}")))?;
            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
                .ok_or_else(|| {
                    AppError::BadRequest(
                        "no primary verified email on github account; please add one".into(),
                    )
                })?
        }
    };

    let github_id = gh_user.id.to_string();

    // Already linked? Sign in directly.
    if let Some(existing) = db::users::find_by_github_id(&state.db, &github_id).await? {
        let token = state
            .jwt
            .issue(&existing.id, &existing.username, existing.is_admin)?;
        return Ok(Json(AuthResponse {
            token,
            user: Some(profile_from_record(&existing)),
        }));
    }

    // Try to link an existing password account by email.
    if let Some(existing) = db::users::find_by_username_or_email(&state.db, &email).await? {
        db::users::link_github_id(&state.db, &existing.id, &github_id).await?;
        let token = state
            .jwt
            .issue(&existing.id, &existing.username, existing.is_admin)?;
        return Ok(Json(AuthResponse {
            token,
            user: Some(profile_from_record(&existing)),
        }));
    }

    // Otherwise create a new account. Password login is disabled for
    // OAuth-only accounts; the hash is a random high-entropy value that
    // verify_password can never match.
    let username = ensure_unique_username(&state.db, &gh_user.login).await?;
    let now = chrono::Utc::now().timestamp();
    let mut random = [0u8; 32];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut random);
    use base64::{Engine as _, engine::general_purpose};
    let placeholder_pw = general_purpose::STANDARD_NO_PAD.encode(random);

    let rec = db::users::UserRecord {
        id: uuid::Uuid::new_v4().to_string(),
        username,
        email,
        display_name: gh_user.name.unwrap_or_default(),
        avatar_url: gh_user.avatar_url.unwrap_or_default(),
        bio: gh_user.bio.unwrap_or_default(),
        password_hash: hash_password(&placeholder_pw)?,
        is_admin: false,
        created_at: now,
        updated_at: now,
        github_id: Some(github_id),
    };
    db::users::insert(&state.db, &rec).await?;
    let token = state.jwt.issue(&rec.id, &rec.username, rec.is_admin)?;
    Ok(Json(AuthResponse {
        token,
        user: Some(profile_from_record(&rec)),
    }))
}

/// If `preferred` is already in use, append `-N` until a free username is found.
async fn ensure_unique_username(
    pool: &crate::db::DbPool,
    preferred: &str,
) -> Result<String, AppError> {
    let base: String = preferred
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let base = if base.is_empty() {
        "gh".to_string()
    } else {
        base
    };
    if db::users::find_by_username_or_email(pool, &base)
        .await?
        .is_none()
    {
        return Ok(base);
    }
    for n in 1..1000 {
        let candidate = format!("{base}-{n}");
        if db::users::find_by_username_or_email(pool, &candidate)
            .await?
            .is_none()
        {
            return Ok(candidate);
        }
    }
    Err(AppError::Conflict(
        "could not pick a unique username".into(),
    ))
}
