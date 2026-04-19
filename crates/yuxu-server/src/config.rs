use anyhow::{Context, Result, bail};
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub live_kit_url: String,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub cors_allowed_origins: Option<String>,
}

impl Config {
    /// Load configuration from environment variables. Fails fast on missing or
    /// malformed values rather than falling back to unsafe defaults (e.g. a
    /// shared dev JWT secret) that would produce silent misbehaviour in
    /// production.
    pub fn from_env() -> Result<Self> {
        let bind_raw = std::env::var("YUXU_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
        let bind: SocketAddr = bind_raw
            .parse()
            .with_context(|| format!("YUXU_BIND is not a valid SocketAddr: {bind_raw}"))?;

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            if cfg!(feature = "postgres") {
                "postgres://postgres:postgres@localhost/yuxu".into()
            } else {
                "sqlite://yuxu.db?mode=rwc".into()
            }
        });

        // JWT secret is required; we never want to accept a built-in fallback,
        // which would let every deployment forge tokens with the same key.
        // `YUXU_DEV_MODE=1` opts in to an ephemeral random secret for local dev.
        let jwt_secret = match std::env::var("YUXU_JWT_SECRET") {
            Ok(s) if s.len() >= 32 => s,
            Ok(_) => bail!("YUXU_JWT_SECRET must be at least 32 bytes"),
            Err(_) => {
                if std::env::var("YUXU_DEV_MODE").ok().as_deref() == Some("1") {
                    use rand::RngCore;
                    let mut buf = [0u8; 48];
                    rand::thread_rng().fill_bytes(&mut buf);
                    use base64::{Engine as _, engine::general_purpose};
                    tracing::warn!(
                        "YUXU_DEV_MODE=1: generated ephemeral JWT secret; tokens won't survive restart"
                    );
                    general_purpose::STANDARD_NO_PAD.encode(buf)
                } else {
                    bail!(
                        "YUXU_JWT_SECRET is required (>=32 bytes); set YUXU_DEV_MODE=1 to auto-generate one for local development"
                    );
                }
            }
        };

        let jwt_ttl_seconds: i64 = match std::env::var("YUXU_JWT_TTL_SECS") {
            Ok(v) => v
                .parse()
                .with_context(|| format!("YUXU_JWT_TTL_SECS must be a positive integer: {v}"))?,
            Err(_) => 60 * 60 * 24,
        };
        if jwt_ttl_seconds <= 0 {
            bail!("YUXU_JWT_TTL_SECS must be positive");
        }

        let live_kit_url = std::env::var("YUXU_LIVEKIT_URL").unwrap_or_default();

        // GitHub OAuth is optional; both pieces must be present together for
        // the /api/auth/github/callback endpoint to function.
        let github_client_id = std::env::var("GITHUB_CLIENT_ID")
            .ok()
            .filter(|s| !s.is_empty());
        let github_client_secret = std::env::var("GITHUB_CLIENT_SECRET")
            .ok()
            .filter(|s| !s.is_empty());
        if github_client_id.is_some() != github_client_secret.is_some() {
            bail!(
                "GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET must be set together (or both unset to disable OAuth)"
            );
        }

        Ok(Self {
            bind,
            database_url,
            jwt_secret,
            jwt_ttl_seconds,
            live_kit_url,
            github_client_id,
            github_client_secret,
            cors_allowed_origins: std::env::var("YUXU_CORS_ORIGINS")
                .ok()
                .filter(|s| !s.is_empty()),
        })
    }
}
