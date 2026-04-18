use crate::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, identifier: &str, credential: &str) -> Result<AuthenticatedUser>;
}
