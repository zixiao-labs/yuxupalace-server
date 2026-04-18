mod client;
mod commands;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yuxu", version, about = "YuXu platform CLI")]
struct Cli {
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
        #[arg(long)]
        password: String,
        #[arg(long)]
        display_name: Option<String>,
    },
    Login {
        #[arg(long)]
        ident: String,
        #[arg(long)]
        password: String,
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
    match cli.cmd {
        Cmd::Auth(AuthCmd::Register {
            username,
            email,
            password,
            display_name,
        }) => commands::auth::register(username, email, password, display_name).await?,
        Cmd::Auth(AuthCmd::Login { ident, password }) => {
            commands::auth::login(ident, password).await?
        }
        Cmd::Auth(AuthCmd::Logout) => commands::auth::logout().await?,
        Cmd::Repo(RepoCmd::List) => commands::repo::list().await?,
        Cmd::Repo(RepoCmd::Create {
            name,
            description,
            private,
        }) => commands::repo::create(name, description, private).await?,
    }
    Ok(())
}
