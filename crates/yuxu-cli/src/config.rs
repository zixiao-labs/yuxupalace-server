use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3000".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default, skip)]
    pub token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".yuxu"))
}

fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn load() -> Result<CliConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(CliConfig::default());
    }
    let content = std::fs::read_to_string(&path).context("failed to read config file")?;
    let config: CliConfig = toml::from_str(&content).context("failed to parse config file")?;
    Ok(config)
}

pub fn save(config: &CliConfig) -> Result<()> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir).context("failed to create config directory")?;
    let content = toml::to_string_pretty(config).context("failed to serialize config")?;
    std::fs::write(config_path()?, content).context("failed to write config file")?;
    Ok(())
}