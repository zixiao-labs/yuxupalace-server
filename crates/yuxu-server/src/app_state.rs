use crate::collab::CollabHub;
use crate::config::Config;
use crate::db::DbPool;
use std::sync::Arc;
use yuxu_core::auth::JwtService;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DbPool,
    pub jwt: Arc<JwtService>,
    pub hub: Arc<CollabHub>,
}
