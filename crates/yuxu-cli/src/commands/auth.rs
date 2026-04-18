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

pub async fn register(
    server: Option<String>,
    username: String,
    email: String,
    password: String,
    display_name: Option<String>,
) -> Result<()> {
    let mut cfg = load_with_override(server)?;
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
    cfg.token = Some(resp.token);
    cfg.username = resp.user.as_ref().map(|u| u.username.clone());
    config::save(&cfg)?;
    println!("registered as {}", cfg.username.as_deref().unwrap_or("?"));
    Ok(())
}

pub async fn login(server: Option<String>, ident: String, password: String) -> Result<()> {
    let mut cfg = load_with_override(server)?;
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
    cfg.token = Some(resp.token);
    cfg.username = resp.user.as_ref().map(|u| u.username.clone());
    config::save(&cfg)?;
    println!("logged in as {}", cfg.username.as_deref().unwrap_or("?"));
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
