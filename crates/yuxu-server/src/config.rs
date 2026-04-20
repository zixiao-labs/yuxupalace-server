use anyhow::{Context, Result, bail};
use std::net::SocketAddr;

/// Where the server is being run. Controls which auth providers are exposed
/// on `/api/auth/config` and which auth endpoints accept traffic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeploymentMode {
    /// Operator-controlled install. Local username/password registration
    /// and login are enabled in addition to OAuth providers.
    SelfHosted,
    /// Hosted multi-tenant install. Local password accounts are disabled;
    /// only OAuth providers (Zixiao Cloud, GitHub) may sign users in.
    Saas,
}

impl DeploymentMode {
    pub fn as_str(self) -> &'static str {
        match self {
            DeploymentMode::SelfHosted => "self-hosted",
            DeploymentMode::Saas => "saas",
        }
    }

    pub fn allows_local_password(self) -> bool {
        matches!(self, DeploymentMode::SelfHosted)
    }
}

/// Standard OAuth 2.0 authorization-code client config. Both id and secret
/// are required to enable the provider; the base URL points at the auth
/// server's public origin.
#[derive(Clone, Debug)]
pub struct ZixiaoCloudConfig {
    pub client_id: String,
    pub client_secret: String,
    pub base_url: String,
}

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub deployment_mode: DeploymentMode,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub live_kit_url: String,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub zixiao_cloud: Option<ZixiaoCloudConfig>,
    pub cors_allowed_origins: Option<String>,
}

// Custom Debug so `tracing::debug!(?config)` or `format!("{:?}", state)` never
// leaks secrets into logs. Fields with credential-shaped content (jwt_secret,
// database_url which may embed a password, github_client_secret,
// zixiao_cloud.client_secret) are elided.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("bind", &self.bind)
            .field("database_url", &"<redacted>")
            .field("deployment_mode", &self.deployment_mode)
            .field("jwt_secret", &"<redacted>")
            .field("jwt_ttl_seconds", &self.jwt_ttl_seconds)
            .field("live_kit_url", &self.live_kit_url)
            .field("github_client_id", &self.github_client_id)
            .field(
                "github_client_secret",
                &self.github_client_secret.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "zixiao_cloud",
                &self.zixiao_cloud.as_ref().map(|z| {
                    format!(
                        "ZixiaoCloudConfig {{ client_id: {:?}, client_secret: <redacted>, base_url: {:?} }}",
                        z.client_id, z.base_url
                    )
                }),
            )
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .finish()
    }
}

/// Read an env var, returning `Some(trimmed)` only when the value contains at
/// least one non-whitespace character. A whitespace-only value is treated as
/// unset so operators don't get silent misbehaviour from stray spaces in
/// `.env` files.
fn env_nonempty_trimmed(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
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

        let deployment_mode = match env_nonempty_trimmed("YUXU_DEPLOYMENT_MODE")
            .map(|v| v.to_ascii_lowercase())
            .as_deref()
        {
            None | Some("self-hosted") | Some("self_hosted") | Some("selfhosted") => {
                DeploymentMode::SelfHosted
            }
            Some("saas") => DeploymentMode::Saas,
            Some(other) => bail!(
                "YUXU_DEPLOYMENT_MODE must be either \"self-hosted\" or \"saas\" (got {other:?})"
            ),
        };

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
        let github_client_id = env_nonempty_trimmed("GITHUB_CLIENT_ID");
        let github_client_secret = env_nonempty_trimmed("GITHUB_CLIENT_SECRET");
        if github_client_id.is_some() != github_client_secret.is_some() {
            bail!(
                "GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET must be set together (or both unset to disable OAuth)"
            );
        }

        // Zixiao Labs Cloud Account OAuth is optional in self-hosted mode but
        // required for any login at all in SaaS mode without GitHub. All three
        // pieces (client_id, client_secret, base_url) must travel together.
        let zixiao_id = env_nonempty_trimmed("ZIXIAO_CLOUD_CLIENT_ID");
        let zixiao_secret = env_nonempty_trimmed("ZIXIAO_CLOUD_CLIENT_SECRET");
        let zixiao_base = env_nonempty_trimmed("ZIXIAO_CLOUD_BASE_URL");
        let zixiao_cloud =
            match (zixiao_id, zixiao_secret, zixiao_base) {
                (Some(client_id), Some(client_secret), Some(mut base_url)) => {
                    while base_url.ends_with('/') {
                        base_url.pop();
                    }
                    Some(ZixiaoCloudConfig {
                        client_id,
                        client_secret,
                        base_url,
                    })
                }
                (None, None, None) => None,
                _ => bail!(
                    "ZIXIAO_CLOUD_CLIENT_ID, ZIXIAO_CLOUD_CLIENT_SECRET and ZIXIAO_CLOUD_BASE_URL must all be set together (or all unset to disable the Zixiao Cloud provider)"
                ),
            };

        if deployment_mode == DeploymentMode::Saas
            && zixiao_cloud.is_none()
            && github_client_id.is_none()
        {
            // SaaS mode disables local password accounts entirely, so an
            // install with no OAuth providers configured cannot accept any
            // user logins. Refuse to start rather than serve a broken /login.
            bail!(
                "YUXU_DEPLOYMENT_MODE=saas requires at least one OAuth provider; configure ZIXIAO_CLOUD_* and/or GITHUB_CLIENT_ID/SECRET"
            );
        }

        Ok(Self {
            bind,
            database_url,
            deployment_mode,
            jwt_secret,
            jwt_ttl_seconds,
            live_kit_url,
            github_client_id,
            github_client_secret,
            zixiao_cloud,
            cors_allowed_origins: env_nonempty_trimmed("YUXU_CORS_ORIGINS"),
        })
    }
}
