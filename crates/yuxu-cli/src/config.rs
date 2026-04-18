use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

fn default_server() -> String {
    "http://localhost:8080".into()
}

/// Resolve the config file path without touching the filesystem. `load()`
/// intentionally does not create the config directory — users on read-only
/// homes would otherwise fail before the missing-file fallback kicks in.
pub fn config_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("no config dir")?
        .join("yuxu")
        .join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config {
            server: default_server(),
            ..Default::default()
        });
    }
    let s = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&s)?)
}

pub fn save(cfg: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = toml::to_string_pretty(cfg)?;

    // The file can carry a bearer token; keep it owner-only on Unix.
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&path)?;
        f.write_all(body.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&path, body)?;
    }
    Ok(())
}
