use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use serde_json::Value;

use crate::client::ApiClient;

#[derive(Subcommand)]
pub enum RepoCmd {
    List,
    Create { #[arg(short, long)] name: String, #[arg(short, long)] description: Option<String>, #[arg(long)] private: bool },
    Info { repo: String },
    Delete { repo: String },
    Branches { repo: String },
}

fn split_repo(repo: &str) -> Result<(&str, &str)> {
    let mut parts = repo.splitn(2, '/');
    let owner = parts.next().ok_or_else(|| anyhow::anyhow!("expected owner/name"))?;
    let name = parts.next().ok_or_else(|| anyhow::anyhow!("expected owner/name"))?;

    if owner.is_empty() || name.is_empty() {
        return Err(anyhow::anyhow!("owner and name must be non-empty"));
    }

    if name.contains('/') {
        return Err(anyhow::anyhow!("expected exactly one '/' in repo string"));
    }

    Ok((owner, name))
}

pub async fn run(cmd: RepoCmd, client: ApiClient) -> Result<()> {
    match cmd {
        RepoCmd::List => {
            let res: Value = client.get("/repos").await?;
            let repos = res["repos"].as_array().cloned().unwrap_or_default();
            for r in &repos {
                println!("{} - {}", r["full_name"].as_str().unwrap_or("").bold(), r["description"].as_str().unwrap_or(""));
            }
            println!("\n{} repositories", repos.len());
        }
        RepoCmd::Create { name, description, private } => {
            let body = serde_json::json!({ "name": name, "description": description.unwrap_or_default(), "is_private": private });
            let res: Value = client.post("/repos", &body).await?;
            println!("Created {}", res["repository"]["full_name"].as_str().unwrap_or(&name).bold());
        }
        RepoCmd::Info { repo } => {
            let (owner, name) = split_repo(&repo)?;
            let res: Value = client.get(&format!("/repos/{}/{}", owner, name)).await?;
            let r = &res["repository"];
            println!("Name:    {}", r["full_name"].as_str().unwrap_or("").bold());
            println!("Private: {}", r["is_private"].as_bool().unwrap_or(false));
            println!("Branch:  {}", r["default_branch"].as_str().unwrap_or("main"));
        }
        RepoCmd::Delete { repo } => {
            let (owner, name) = split_repo(&repo)?;
            client.delete(&format!("/repos/{}/{}", owner, name)).await?;
            println!("Deleted {}", repo.bold());
        }
        RepoCmd::Branches { repo } => {
            let (owner, name) = split_repo(&repo)?;
            let res: Value = client.get(&format!("/repos/{}/{}/branches", owner, name)).await?;
            for b in res["branches"].as_array().cloned().unwrap_or_default() {
                println!("  {}", b.as_str().unwrap_or(""));
            }
        }
    }
    Ok(())
}