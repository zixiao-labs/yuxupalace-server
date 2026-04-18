use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::Serialize;

pub struct Client {
    http: reqwest::Client,
    server: String,
    token: Option<String>,
}

impl Client {
    pub fn new(cfg: &Config) -> Self {
        Self {
            http: reqwest::Client::new(),
            server: cfg.server.clone(),
            token: cfg.token.clone(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Some(t) = &self.token
            && let Ok(v) = HeaderValue::from_str(&format!("Bearer {t}"))
        {
            h.insert(AUTHORIZATION, v);
        }
        h
    }

    pub async fn post<B: Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R> {
        let resp = self
            .http
            .post(format!("{}{}", self.server, path))
            .headers(self.headers())
            .json(body)
            .send()
            .await
            .context("http post")?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            anyhow::bail!("{}: {}", status, String::from_utf8_lossy(&bytes));
        }
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub async fn get<R: serde::de::DeserializeOwned>(&self, path: &str) -> Result<R> {
        let resp = self
            .http
            .get(format!("{}{}", self.server, path))
            .headers(self.headers())
            .send()
            .await
            .context("http get")?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            anyhow::bail!("{}: {}", status, String::from_utf8_lossy(&bytes));
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
}
