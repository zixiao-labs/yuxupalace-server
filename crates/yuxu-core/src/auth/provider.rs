use crate::Result;
use async_trait::async_trait;

/// Identity returned by an `AuthProvider`. `Debug` is implemented manually so
/// that emails (PII) are never leaked through `{:?}` logging.
#[derive(Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
}

impl std::fmt::Debug for AuthenticatedUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthenticatedUser")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &"<redacted>")
            .field("is_admin", &self.is_admin)
            .finish()
    }
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, identifier: &str, credential: &str) -> Result<AuthenticatedUser>;
}
