use crate::ws::CollabRoom;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub git_root: PathBuf,
    pub jwt_secret: String,
    pub collab_rooms: Arc<DashMap<String, CollabRoom>>,
}

impl AppState {
    pub fn new(db: sqlx::PgPool, git_root: PathBuf, jwt_secret: String) -> Self {
        Self {
            db,
            git_root,
            jwt_secret,
            collab_rooms: Arc::new(DashMap::new()),
        }
    }
}
