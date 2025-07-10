use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Duration;
use jsonwebtoken::DecodingKey;
use reqwest::{Client, ClientBuilder, Response, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::error::WazuhError;

#[derive(Debug, Clone)]
pub struct WazuhClient {
    client: Client,
    config: Arc<RwLock<Config>>,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    data: WazuhTokenData,
}

#[derive(Debug, Deserialize)]
struct WazuhTokenData {
    token: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    error: i32,
    message: String,
}

impl WazuhClient {
    /// Create a new Wazuh API client
    pub async fn new(config: Arc<RwLock<Config>>) -> Result<Self> {
        let cfg = config.read().await;
        let base_url = cfg.api_url();
        
        let mut client_builder = ClientBuilder::new()
            .timeout(StdDuration::from_secs(cfg.api.timeout))
            .danger_accept_invalid_certs(!cfg.tls.verify);

        // Add custom CA certificate if provided
        if let Some(ca_cert_path) = &cfg.tls.ca_cert {
            let cert = std::fs::read(ca_cert_path)
                .with_context(|| format!("Failed to read CA certificate: {:?}", ca_cert_path))?;
            let cert = reqwest::Certificate::from_pem(&cert)
                .context("Failed to parse CA certificate")?;
            client_builder = client_builder.add_root_certificate(cert);
        }

        // Add client certificate if provided
        if let (Some(cert_path), Some(key_path)) = (&cfg.tls.client_cert, &cfg.tls.client_key) {
            let cert = std::fs::read(cert_path)
                .with_context(|| format!("Failed to read client certificate: {:?}", cert_path))?;
            let key = std::fs::read(key_path)
                .with_context(|| format!("Failed to read client key: {:?}", key_path))?;
            
            let identity = reqwest::Identity::from_pem(&[cert, key].concat())
                .context("Failed to create client identity")?;
            client_builder = client_builder.identity(identity);
        }

        let client = client_builder.build()
            .context("Failed to build HTTP client")?;

        drop(cfg); // Release the read lock

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    /// Authenticate with the Wazuh API
    pub async fn authenticate(&self) -> Result<()> {
        let mut config = self.config.write().await;
        
        // Check if we already have a valid token
        if let Some(token) = &config.auth.token {
            if self.is_token_valid(token).await? {
                info!("Using existing valid token");
                return Ok(());
            }
        }

        // Get credentials
        let (username, password) = match (&config.auth.username, &config.auth.password) {
            (Some(u), Some(p)) => (u.clone(), p.clone()),
            _ => return Err(anyhow!("Username and password required for authentication")),
        };

        drop(config); // Release write lock before making request

        // Make login request
        let login_url = format!("{}/security/user/authenticate", self.base_url);
        let auth_header = format!("Basic {}", BASE64.encode(format!("{}:{}", username, password)));
        
        debug!("Authenticating with Wazuh API at: {}", login_url);
        
        let response = self.client
            .post(&login_url)
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to send authentication request")?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            let error: ApiError = serde_json::from_str(&body)
                .unwrap_or(ApiError {
                    error: status.as_u16() as i32,
                    message: body,
                });
            return Err(WazuhError::ApiError {
                code: error.error,
                message: error.message,
            }.into());
        }

        let login_response: LoginResponse = serde_json::from_str(&body)
            .context("Failed to parse login response")?;

        // Update config with new token
        let mut config = self.config.write().await;
        config.update_token(login_response.data.token);
        
        info!("Successfully authenticated with Wazuh API");
        Ok(())
    }

    /// Check if a token is still valid
    async fn is_token_valid(&self, _token: &str) -> Result<bool> {
        // In a real implementation, you would decode the JWT and check expiration
        // For now, we'll do a simple test request
        let test_url = format!("{}/security/user/authenticate/run_as", self.base_url);
        let response = self.get(&test_url).await?;
        Ok(response.status() != StatusCode::UNAUTHORIZED)
    }

    /// Make a GET request to the API
    pub async fn get(&self, endpoint: &str) -> Result<Response> {
        self.request(reqwest::Method::GET, endpoint, None::<()>).await
    }

    /// Make a POST request to the API
    pub async fn post<T: Serialize>(&self, endpoint: &str, body: Option<T>) -> Result<Response> {
        self.request(reqwest::Method::POST, endpoint, body).await
    }

    /// Make a PUT request to the API
    pub async fn put<T: Serialize>(&self, endpoint: &str, body: Option<T>) -> Result<Response> {
        self.request(reqwest::Method::PUT, endpoint, body).await
    }

    /// Make a DELETE request to the API
    pub async fn delete(&self, endpoint: &str) -> Result<Response> {
        self.request(reqwest::Method::DELETE, endpoint, None::<()>).await
    }

    /// Make a generic request to the API
    async fn request<T: Serialize>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<T>,
    ) -> Result<Response> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let config = self.config.read().await;
        let token = config.auth.token.as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        let mut request = self.client
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", token));

        if let Some(ref body) = body {
            request = request
                .header("Content-Type", "application/json")
                .json(body);
        }

        drop(config); // Release read lock

        debug!("{} {}", method, url);
        
        let response = request.send().await
            .with_context(|| format!("Failed to send {} request to {}", method, url))?;

        // Handle authentication errors by trying to re-authenticate once
        if response.status() == StatusCode::UNAUTHORIZED {
            warn!("Token expired, attempting to re-authenticate");
            self.authenticate().await?;
            
            // Retry the request with new token
            let config = self.config.read().await;
            let token = config.auth.token.as_ref()
                .ok_or_else(|| anyhow!("Failed to get new token"))?;

            let mut request = self.client
                .request(method.clone(), &url)
                .header("Authorization", format!("Bearer {}", token));

            if let Some(ref body) = body {
                request = request
                    .header("Content-Type", "application/json")
                    .json(body);
            }

            drop(config);

            return request.send().await
                .with_context(|| format!("Failed to retry {} request to {}", method, url));
        }

        Ok(response)
    }

    /// Parse JSON response from the API
    pub async fn parse_response<T: DeserializeOwned>(response: Response) -> Result<T> {
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            let error: ApiError = serde_json::from_str(&body)
                .unwrap_or(ApiError {
                    error: status.as_u16() as i32,
                    message: body,
                });
            return Err(WazuhError::ApiError {
                code: error.error,
                message: error.message,
            }.into());
        }

        serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse response: {}", body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_client_creation() {
        let config = Arc::new(RwLock::new(Config::default()));
        let client = WazuhClient::new(config).await;
        assert!(client.is_ok());
    }
}