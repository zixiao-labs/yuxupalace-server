mod client;
mod commands;
mod config;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yuxu", about = "YuXu DevOps CLI", version)]
struct Cli {
    /// Server URL (overrides config)
    #[arg(long, global = true, env = "YUXU_SERVER")]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        cmd: commands::auth::AuthCmd,
    },
    /// Repository commands
    Repo {
        #[command(subcommand)]
        cmd: commands::repo::RepoCmd,
    },
    /// Issue commands
    Issue {
        #[command(subcommand)]
        cmd: commands::issue::IssueCmd,
    },
    /// Merge request commands
    Mr {
        #[command(subcommand)]
        cmd: commands::mr::MrCmd,
    },
    /// Pipeline commands
    Pipeline {
        #[command(subcommand)]
        cmd: commands::pipeline::PipelineCmd,
    },
    /// Member management commands
    Member {
        #[command(subcommand)]
        cmd: commands::member::MemberCmd,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut cfg = config::load().unwrap_or_else(|_| config::CliConfig::default());
    if let Some(server) = cli.server {
        cfg.server.url = server;
    }

    let client = client::ApiClient::new(&cfg)?;

    let result = match cli.command {
        Commands::Auth { cmd } => commands::auth::run(cmd, client, &mut cfg).await,
        Commands::Repo { cmd } => commands::repo::run(cmd, client).await,
        Commands::Issue { cmd } => commands::issue::run(cmd, client).await,
        Commands::Mr { cmd } => commands::mr::run(cmd, client).await,
        Commands::Pipeline { cmd } => commands::pipeline::run(cmd, client).await,
        Commands::Member { cmd } => commands::member::run(cmd, client).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}