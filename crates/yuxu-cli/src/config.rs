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

pub fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().context("no config dir")?.join("yuxu");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("config.toml"))
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
    std::fs::write(&path, toml::to_string_pretty(cfg)?)?;
    Ok(())
}
