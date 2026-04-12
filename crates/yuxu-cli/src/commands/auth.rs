use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use serde_json::Value;

use crate::client::ApiClient;
use crate::config::{self, CliConfig};

#[derive(Subcommand)]
pub enum AuthCmd {
    Login {
        #[arg(short, long)]
        username: String,
        password: Option<String>,
    },
    Register {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        email: String,
        password: Option<String>,
        #[arg(long)]
        display_name: Option<String>,
    },
    Whoami,
    Logout,
}

pub async fn run(cmd: AuthCmd, client: ApiClient, cfg: &mut CliConfig) -> Result<()> {
    match cmd {
        AuthCmd::Login { username, password } => {
            let password = match password {
                Some(p) => p,
                None => {
                    println!("Password: ");
                    rpassword::read_password()?
                }
            };
            let body = serde_json::json!({ "username": username, "password": password });
            let res: Value = client.post("/auth/login", &body).await?;
            let token = res["token"].as_str()
                .filter(|t| !t.is_empty())
                .ok_or_else(|| anyhow::anyhow!("server returned empty or missing token"))?
                .to_string();
            let uname = res["user"]["username"].as_str().unwrap_or(&username).to_string();
            cfg.auth.token = Some(token);
            cfg.auth.username = Some(uname.clone());
            config::save(cfg)?;
            println!("Logged in as {}", uname.bold());
        }
        AuthCmd::Register { username, email, password, display_name } => {
            let password = match password {
                Some(p) => p,
                None => {
                    println!("Password: ");
                    rpassword::read_password()?
                }
            };
            let body = serde_json::json!({ "username": username, "email": email, "password": password, "display_name": display_name });
            let res: Value = client.post("/auth/register", &body).await?;
            let token = res["token"].as_str()
                .filter(|t| !t.is_empty())
                .ok_or_else(|| anyhow::anyhow!("server returned empty or missing token"))?
                .to_string();
            cfg.auth.token = Some(token);
            cfg.auth.username = Some(username.clone());
            config::save(cfg)?;
            println!("Registered and logged in as {}", username.bold());
        }
        AuthCmd::Whoami => {
            let res: Value = client.get("/auth/me").await?;
            let user = &res["user"];
            println!("Username: {}", user["username"].as_str().unwrap_or("").bold());
            println!("Email:    {}", user["email"].as_str().unwrap_or(""));
            println!("Admin:    {}", user["is_admin"].as_bool().unwrap_or(false));
        }
        AuthCmd::Logout => {
            cfg.auth.token = None;
            cfg.auth.username = None;
            config::save(cfg)?;
            println!("Logged out");
        }
    }
    Ok(())
}