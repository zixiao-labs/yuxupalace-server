use crate::{client::Client, config};
use anyhow::Result;
use raidian::{AuthResponse, LoginRequest, RegisterRequest};

fn load_with_override(server: Option<String>) -> Result<config::Config> {
    let mut cfg = config::load()?;
    if let Some(s) = server {
        cfg.server = s;
    }
    Ok(cfg)
}

/// Persist `token`/`username` from an auth response without turning a
/// `--server` override into a sticky config entry. The override is documented
/// as per-invocation, so we reload the on-disk config before saving.
fn persist_auth(token: Option<String>, username: Option<String>) -> Result<()> {
    let mut disk = config::load()?;
    disk.token = token;
    disk.username = username;
    config::save(&disk)
}

pub async fn register(
    server: Option<String>,
    username: String,
    email: String,
    password: String,
    display_name: Option<String>,
) -> Result<()> {
    let cfg = load_with_override(server)?;
    let client = Client::new(&cfg);
    let resp: AuthResponse = client
        .post(
            "/api/auth/register",
            &RegisterRequest {
                username: username.clone(),
                email,
                password,
                display_name: display_name.unwrap_or_else(|| username.clone()),
            },
        )
        .await?;
    let saved_username = resp.user.as_ref().map(|u| u.username.clone());
    persist_auth(Some(resp.token), saved_username.clone())?;
    println!("registered as {}", saved_username.as_deref().unwrap_or("?"));
    Ok(())
}

pub async fn login(server: Option<String>, ident: String, password: String) -> Result<()> {
    let cfg = load_with_override(server)?;
    let client = Client::new(&cfg);
    let resp: AuthResponse = client
        .post(
            "/api/auth/login",
            &LoginRequest {
                username_or_email: ident,
                password,
            },
        )
        .await?;
    let saved_username = resp.user.as_ref().map(|u| u.username.clone());
    persist_auth(Some(resp.token), saved_username.clone())?;
    println!("logged in as {}", saved_username.as_deref().unwrap_or("?"));
    Ok(())
}

pub async fn logout() -> Result<()> {
    let mut cfg = config::load()?;
    cfg.token = None;
    cfg.username = None;
    config::save(&cfg)?;
    println!("logged out");
    Ok(())
}
