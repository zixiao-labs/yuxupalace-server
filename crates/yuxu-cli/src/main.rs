mod client;
mod commands;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yuxu", version, about = "YuXu platform CLI")]
struct Cli {
    /// Override the API server URL for this invocation. Bootstraps a first
    /// connection to a remote/self-hosted backend without editing config.toml
    /// by hand.
    #[arg(long, global = true, env = "YUXU_SERVER")]
    server: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(subcommand)]
    Auth(AuthCmd),
    #[command(subcommand)]
    Repo(RepoCmd),
}

#[derive(Subcommand)]
enum AuthCmd {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        /// Password. Prompted interactively if omitted; passing it on the CLI
        /// leaks it into shell history and `ps aux`.
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        display_name: Option<String>,
    },
    Login {
        #[arg(long)]
        ident: String,
        /// Password. Prompted interactively if omitted.
        #[arg(long)]
        password: Option<String>,
    },
    Logout,
}

#[derive(Subcommand)]
enum RepoCmd {
    List,
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value_t = false)]
        private: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let cli = Cli::parse();
    let server_override = cli.server.clone();
    match cli.cmd {
        Cmd::Auth(AuthCmd::Register {
            username,
            email,
            password,
            display_name,
        }) => {
            let password = prompt_password_if_missing(password, "Password: ")?;
            commands::auth::register(server_override, username, email, password, display_name)
                .await?
        }
        Cmd::Auth(AuthCmd::Login { ident, password }) => {
            let password = prompt_password_if_missing(password, "Password: ")?;
            commands::auth::login(server_override, ident, password).await?
        }
        Cmd::Auth(AuthCmd::Logout) => commands::auth::logout().await?,
        Cmd::Repo(RepoCmd::List) => commands::repo::list(server_override).await?,
        Cmd::Repo(RepoCmd::Create {
            name,
            description,
            private,
        }) => commands::repo::create(server_override, name, description, private).await?,
    }
    Ok(())
}

fn prompt_password_if_missing(pw: Option<String>, prompt: &str) -> Result<String> {
    match pw {
        Some(p) => Ok(p),
        None => Ok(rpassword::prompt_password(prompt)?),
    }
}
