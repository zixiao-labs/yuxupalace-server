use crate::{client::Client, config};
use anyhow::Result;
use raidian::{CreateRepositoryRequest, Repository};

fn load_with_override(server: Option<String>) -> Result<config::Config> {
    let mut cfg = config::load()?;
    if let Some(s) = server {
        cfg.server = s;
    }
    Ok(cfg)
}

pub async fn list(server: Option<String>) -> Result<()> {
    let cfg = load_with_override(server)?;
    let client = Client::new(&cfg);
    let repos: Vec<Repository> = client.get("/api/repos").await?;
    for r in repos {
        println!(
            "{}\t{}\t{}",
            r.full_name,
            if r.is_private { "private" } else { "public" },
            r.description
        );
    }
    Ok(())
}

pub async fn create(
    server: Option<String>,
    name: String,
    description: Option<String>,
    private: bool,
) -> Result<()> {
    let cfg = load_with_override(server)?;
    let client = Client::new(&cfg);
    let r: Repository = client
        .post(
            "/api/repos",
            &CreateRepositoryRequest {
                name,
                description: description.unwrap_or_default(),
                is_private: private,
            },
        )
        .await?;
    println!("created: {}", r.full_name);
    Ok(())
}
