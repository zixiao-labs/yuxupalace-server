use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

use crate::config::CliConfig;

pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.error, self.code)
    }
}

impl ApiClient {
    pub fn new(config: &CliConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to initialize HTTP client")?;

        Ok(Self {
            client,
            base_url: config.server.url.trim_end_matches('/').to_string(),
            token: config.auth.token.clone(),
        })
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let mut req = self.client.get(self.url(path));
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let response = req.send().await.context("request failed")?;
        self.handle_response(response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let mut req = self.client.post(self.url(path)).json(body);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let response = req.send().await.context("request failed")?;
        self.handle_response(response).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let mut req = self.client.put(self.url(path)).json(body);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let response = req.send().await.context("request failed")?;
        self.handle_response(response).await
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let mut req = self.client.delete(self.url(path));
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let response = req.send().await.context("request failed")?;
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            // Try to parse as JSON first
            let body_bytes = response.bytes().await.context("failed to read response body")?;
            match serde_json::from_slice::<ApiError>(&body_bytes) {
                Ok(api_err) => Err(anyhow::anyhow!("{}", api_err)),
                Err(_) => {
                    // Fall back to raw body text if JSON parse fails
                    let body_text = String::from_utf8_lossy(&body_bytes);
                    Err(anyhow::anyhow!(
                        "request failed with status {}: {}",
                        status,
                        body_text
                    ))
                }
            }
        }
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        if response.status().is_success() {
            response.json().await.context("failed to parse response")
        } else {
            let status = response.status();
            // Try to parse as JSON first
            let body_bytes = response.bytes().await.context("failed to read response body")?;
            match serde_json::from_slice::<ApiError>(&body_bytes) {
                Ok(api_err) => Err(anyhow::anyhow!("{}", api_err)),
                Err(_) => {
                    // Fall back to raw body text if JSON parse fails
                    let body_text = String::from_utf8_lossy(&body_bytes);
                    Err(anyhow::anyhow!(
                        "request failed with status {}: {}",
                        status,
                        body_text
                    ))
                }
            }
        }
    }
}