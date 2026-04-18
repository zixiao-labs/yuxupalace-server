use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub live_kit_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let bind = std::env::var("YUXU_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
        let bind: SocketAddr = bind.parse().expect("YUXU_BIND is not a valid SocketAddr");

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            if cfg!(feature = "postgres") {
                "postgres://postgres:postgres@localhost/yuxu".into()
            } else {
                "sqlite://yuxu.db?mode=rwc".into()
            }
        });

        let jwt_secret = std::env::var("YUXU_JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-change-me-dev-secret-change-me".into());

        let jwt_ttl_seconds: i64 = std::env::var("YUXU_JWT_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60 * 60 * 24);

        let live_kit_url = std::env::var("YUXU_LIVEKIT_URL").unwrap_or_default();

        Self {
            bind,
            database_url,
            jwt_secret,
            jwt_ttl_seconds,
            live_kit_url,
        }
    }
}
