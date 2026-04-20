use crate::{app_state::AppState, db, error::AppError, middleware::auth::AuthUser};
use axum::{Json, extract::State};
use base64::{Engine as _, engine::general_purpose};
use raidian::{AuthResponse, GithubOauthRequest, LoginRequest, RegisterRequest, UserProfile};
use rand::RngCore;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
pub struct AuthConfigResponse {
    pub deployment_mode: &'static str,
    pub providers: AuthProviders,
}

#[derive(Serialize)]
pub struct AuthProviders {
    pub local: bool,
    pub github: bool,
    pub zixiao_cloud: Option<ZixiaoCloudClientConfig>,
}

/// Public OAuth client parameters the browser needs to build the authorize
/// redirect. `client_secret` is deliberately omitted.
#[derive(Serialize)]
pub struct ZixiaoCloudClientConfig {
    pub client_id: String,
    pub base_url: String,
}

/// Expose deployment mode + enabled auth providers so the frontend can render
/// the correct login surface without having to duplicate the env-var rules.
/// Reachable without authentication — it intentionally leaks no secrets.
pub async fn config(State(state): State<AppState>) -> Json<AuthConfigResponse> {
    let cfg = &state.config;
    Json(AuthConfigResponse {
        deployment_mode: cfg.deployment_mode.as_str(),
        providers: AuthProviders {
            local: cfg.deployment_mode.allows_local_password(),
            github: cfg.github_client_id.is_some(),
            zixiao_cloud: cfg.zixiao_cloud.as_ref().map(|z| ZixiaoCloudClientConfig {
                client_id: z.client_id.clone(),
                base_url: z.base_url.clone(),
            }),
        },
    })
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if !state.config.deployment_mode.allows_local_password() {
        return Err(AppError::Forbidden(
            "local account registration is disabled in SaaS mode".into(),
        ));
    }
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
        zixiao_cloud_id: None,
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
    if !state.config.deployment_mode.allows_local_password() {
        return Err(AppError::Forbidden(
            "local password login is disabled in SaaS mode".into(),
        ));
    }
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

    let http = &state.http;

    let token_resp: GithubAccessTokenResp = send_github_json(
        http.post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("code", req.code.as_str()),
            ]),
        "github token exchange",
    )
    .await?;

    if let Some(err) = token_resp.error {
        let desc = token_resp.error_description.unwrap_or_default();
        tracing::warn!(%err, %desc, "github returned oauth error");
        return Err(AppError::Unauthorized("github oauth failed".into()));
    }
    let access_token = token_resp
        .access_token
        .ok_or_else(|| AppError::Unauthorized("github oauth failed".into()))?;

    let gh_user: GithubUser = send_github_json(
        http.get("https://api.github.com/user")
            .bearer_auth(&access_token)
            .header("Accept", "application/vnd.github+json"),
        "github /user",
    )
    .await?;

    // Always resolve the identity via /user/emails: gh_user.email is the
    // user's *publicly visible* address which may not be verified. Requiring
    // primary && verified here keeps the conflict check and account creation
    // tied to an email GitHub has actually proven the user controls.
    let emails: Vec<GithubEmail> = send_github_json(
        http.get("https://api.github.com/user/emails")
            .bearer_auth(&access_token)
            .header("Accept", "application/vnd.github+json"),
        "github /user/emails",
    )
    .await?;
    let email = emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
        .ok_or_else(|| {
            AppError::BadRequest(
                "no primary verified email on github account; please add one".into(),
            )
        })?;

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

    // Refuse to auto-link an existing yuxu account just because the GitHub
    // email claim matches. GitHub verifies that the user controls the email
    // *at GitHub*, not that they own the pre-existing yuxu account using the
    // same address — so auto-linking would let anyone who can sign up at
    // GitHub with a targeted email take over a password account. The user
    // must sign in with their existing credentials and opt in to linking
    // GitHub from account settings (TODO: expose that endpoint).
    if db::users::find_by_username_or_email(&state.db, &email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(
            "an account with this email already exists; sign in with your password, then link GitHub from settings".into(),
        ));
    }

    // Otherwise create a new account. Password login is disabled for
    // OAuth-only accounts; the hash is a random high-entropy value that
    // verify_password can never match.
    let username = ensure_unique_username(&state.db, &gh_user.login).await?;
    let now = chrono::Utc::now().timestamp();
    let mut random = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random);
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
        github_id: Some(github_id.clone()),
        zixiao_cloud_id: None,
    };
    if let Err(err) = db::users::insert(&state.db, &rec).await {
        // If a concurrent callback for the same GitHub id raced us past
        // find_by_github_id and inserted first, recover by re-reading and
        // signing in as that user instead of returning a 500. Any other
        // unique-violation (email, username) means a genuine conflict we
        // can't safely paper over, so propagate the error.
        if matches!(&err, AppError::Sqlx(e) if is_unique_violation_on(e, "github_id"))
            && let Some(existing) = db::users::find_by_github_id(&state.db, &github_id).await?
        {
            let token = state
                .jwt
                .issue(&existing.id, &existing.username, existing.is_admin)?;
            return Ok(Json(AuthResponse {
                token,
                user: Some(profile_from_record(&existing)),
            }));
        }
        return Err(err);
    }
    let token = state.jwt.issue(&rec.id, &rec.username, rec.is_admin)?;
    Ok(Json(AuthResponse {
        token,
        user: Some(profile_from_record(&rec)),
    }))
}

/// Detect a unique-constraint violation on the column whose name appears in
/// the DB's error message. Works across sqlite (error code "2067", message
/// "UNIQUE constraint failed: users.github_id") and postgres (SQLSTATE
/// "23505", constraint name like "idx_users_github_id"). The column-hint
/// check is a substring match on the message so we only recover from the
/// specific collision we intended to.
fn is_unique_violation_on(err: &sqlx::Error, column_hint: &str) -> bool {
    let Some(db_err) = err.as_database_error() else {
        return false;
    };
    let unique = matches!(db_err.code().as_deref(), Some("2067") | Some("23505"));
    unique && db_err.message().contains(column_hint)
}

/// Check HTTP status before attempting JSON decode so a 4xx/5xx from GitHub
/// surfaces as an HTTP error with the real body snippet, not as a misleading
/// "missing field `id`" JSON decode failure.
///
/// Note: GitHub's OAuth token-exchange endpoint intentionally returns 200 with
/// an `error` field in the JSON body on application-level failures, so the
/// caller still needs to inspect that field after decode.
async fn send_github_json<T: serde::de::DeserializeOwned>(
    req: reqwest::RequestBuilder,
    ctx: &'static str,
) -> Result<T, AppError> {
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("{ctx}: {e}")))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(500).collect();
        tracing::warn!(%status, %snippet, "{} returned non-success", ctx);
        return Err(AppError::Anyhow(anyhow::anyhow!(
            "{ctx}: HTTP {status}: {snippet}"
        )));
    }
    resp.json()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("decode {ctx}: {e}")))
}

/// If `preferred` is already in use, append `-N` until a free username is found.
async fn ensure_unique_username(
    pool: &crate::db::DbPool,
    preferred: &str,
) -> Result<String, AppError> {
    // GitHub caps logins at 39 chars but we also want suffix room for `-N`
    // collision-breaking and a hard limit against pathological inputs.
    const MAX_BASE_LEN: usize = 32;
    let sanitized: String = preferred
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(MAX_BASE_LEN)
        .collect();
    let base = if sanitized.is_empty() {
        "gh".to_string()
    } else {
        sanitized
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

#[derive(Deserialize)]
pub struct ZixiaoOauthRequest {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
    /// Frontend-supplied redirect_uri that was used when starting the
    /// authorization-code flow. Forwarded to the token endpoint verbatim —
    /// the provider enforces an exact match.
    pub redirect_uri: String,
}

#[derive(Deserialize)]
struct ZixiaoTokenResp {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct ZixiaoUserInfo {
    /// Stable subject identifier; required.
    sub: String,
    username: Option<String>,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
}

/// OAuth 2.0 authorization-code callback for the Zixiao Labs Cloud Account
/// provider. The browser runs the authorize redirect itself using the
/// `base_url` + `client_id` it discovered from `/api/auth/config`, then POSTs
/// the returned code here. We exchange the code for an access token, fetch
/// the userinfo document, and upsert a local account keyed on `sub`.
pub async fn zixiao_callback(
    State(state): State<AppState>,
    Json(req): Json<ZixiaoOauthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let zx = state
        .config
        .zixiao_cloud
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("zixiao cloud oauth not configured".into()))?;
    if req.code.trim().is_empty() {
        return Err(AppError::BadRequest("missing code".into()));
    }
    if req.redirect_uri.trim().is_empty() {
        return Err(AppError::BadRequest("missing redirect_uri".into()));
    }

    let http = &state.http;

    let token_url = format!("{}/oauth/token", zx.base_url);
    let token_resp: ZixiaoTokenResp = send_oauth_json(
        http.post(&token_url)
            .header("Accept", "application/json")
            .basic_auth(&zx.client_id, Some(&zx.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", req.code.as_str()),
                ("redirect_uri", req.redirect_uri.as_str()),
                ("client_id", zx.client_id.as_str()),
            ]),
        "zixiao token exchange",
    )
    .await?;

    if let Some(err) = token_resp.error {
        let desc = token_resp.error_description.unwrap_or_default();
        tracing::warn!(%err, %desc, "zixiao returned oauth error");
        return Err(AppError::Unauthorized("zixiao oauth failed".into()));
    }
    let access_token = token_resp
        .access_token
        .ok_or_else(|| AppError::Unauthorized("zixiao oauth failed".into()))?;

    let userinfo: ZixiaoUserInfo = send_oauth_json(
        http.get(format!("{}/oauth/userinfo", zx.base_url))
            .bearer_auth(&access_token)
            .header("Accept", "application/json"),
        "zixiao /oauth/userinfo",
    )
    .await?;

    let sub = userinfo.sub.trim().to_string();
    if sub.is_empty() {
        return Err(AppError::Unauthorized(
            "zixiao userinfo missing subject".into(),
        ));
    }

    if let Some(existing) = db::users::find_by_zixiao_cloud_id(&state.db, &sub).await? {
        let token = state
            .jwt
            .issue(&existing.id, &existing.username, existing.is_admin)?;
        return Ok(Json(AuthResponse {
            token,
            user: Some(profile_from_record(&existing)),
        }));
    }

    // Don't auto-link an existing local account just because emails match —
    // same reasoning as in `github_callback`: the provider has only proven the
    // user controls *that provider's* view of the email, not the local
    // account with the same address.
    let email = userinfo.email.unwrap_or_default();
    if !email.is_empty()
        && db::users::find_by_username_or_email(&state.db, &email)
            .await?
            .is_some()
    {
        return Err(AppError::Conflict(
            "an account with this email already exists; sign in and link the cloud account from settings"
                .into(),
        ));
    }

    let preferred_username = userinfo.username.clone().unwrap_or_else(|| sub.clone());
    let username = ensure_unique_username(&state.db, &preferred_username).await?;
    let now = chrono::Utc::now().timestamp();
    let mut random = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random);
    let placeholder_pw = general_purpose::STANDARD_NO_PAD.encode(random);

    let rec = db::users::UserRecord {
        id: uuid::Uuid::new_v4().to_string(),
        username,
        email,
        display_name: userinfo.name.unwrap_or_default(),
        avatar_url: userinfo.avatar_url.unwrap_or_default(),
        bio: userinfo.bio.unwrap_or_default(),
        password_hash: hash_password(&placeholder_pw)?,
        is_admin: false,
        created_at: now,
        updated_at: now,
        github_id: None,
        zixiao_cloud_id: Some(sub.clone()),
    };
    if let Err(err) = db::users::insert(&state.db, &rec).await {
        if matches!(&err, AppError::Sqlx(e) if is_unique_violation_on(e, "zixiao_cloud_id"))
            && let Some(existing) = db::users::find_by_zixiao_cloud_id(&state.db, &sub).await?
        {
            let token = state
                .jwt
                .issue(&existing.id, &existing.username, existing.is_admin)?;
            return Ok(Json(AuthResponse {
                token,
                user: Some(profile_from_record(&existing)),
            }));
        }
        return Err(err);
    }
    let token = state.jwt.issue(&rec.id, &rec.username, rec.is_admin)?;
    Ok(Json(AuthResponse {
        token,
        user: Some(profile_from_record(&rec)),
    }))
}

/// Like `send_github_json` but tolerant of standards-compliant OAuth 2.0
/// error responses. RFC 6749 §5.2 mandates that token-endpoint failures come
/// back as HTTP 4xx with a JSON body such as `{"error": "invalid_grant"}`;
/// callers (e.g. `zixiao_callback`) rely on decoding that body into a struct
/// with optional `error`/`error_description` fields to surface the real
/// reason. Only if the body isn't the expected shape do we fall through to a
/// generic transport-style AppError.
async fn send_oauth_json<T: serde::de::DeserializeOwned>(
    req: reqwest::RequestBuilder,
    ctx: &'static str,
) -> Result<T, AppError> {
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("{ctx}: {e}")))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        match serde_json::from_str::<T>(&body) {
            Ok(decoded) => return Ok(decoded),
            Err(decode_err) => {
                let snippet: String = body.chars().take(500).collect();
                tracing::warn!(
                    %status,
                    %snippet,
                    decode_err = %decode_err,
                    "{} returned non-success and body did not decode",
                    ctx,
                );
                return Err(AppError::Anyhow(anyhow::anyhow!(
                    "{ctx}: HTTP {status}: {snippet}"
                )));
            }
        }
    }
    resp.json()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("decode {ctx}: {e}")))
}
