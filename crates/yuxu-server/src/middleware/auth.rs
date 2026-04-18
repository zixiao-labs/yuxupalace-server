use crate::app_state::AppState;
use crate::error::AppError;
use axum::{extract::FromRequestParts, http::request::Parts};
use yuxu_core::auth::Claims;

pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app: AppState = axum::extract::FromRef::from_ref(state);
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing credentials".into()))?;

        // Accept any case of the `Bearer` scheme and tolerate surrounding
        // whitespace. Intentionally use a single generic error message so we
        // don't leak whether the scheme, token, or signature was at fault.
        let generic = || AppError::Unauthorized("invalid credentials".into());
        let (scheme, token) = header.split_once(' ').ok_or_else(generic)?;
        if !scheme.eq_ignore_ascii_case("Bearer") {
            return Err(generic());
        }
        let token = token.trim();
        if token.is_empty() {
            return Err(generic());
        }
        let claims = app.jwt.verify(token).map_err(|e| {
            tracing::debug!(error = %e, "jwt verification failed");
            generic()
        })?;
        Ok(AuthUser(claims))
    }
}
