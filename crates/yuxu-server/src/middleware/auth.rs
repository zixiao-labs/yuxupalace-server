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
            .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("expected `Bearer <token>`".into()))?;
        let claims = app
            .jwt
            .verify(token)
            .map_err(|e| AppError::Unauthorized(e.to_string()))?;
        Ok(AuthUser(claims))
    }
}
