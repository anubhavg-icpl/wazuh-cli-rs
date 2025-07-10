use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_FILE: &str = "config.toml";
const APP_NAME: &str = "wazuh-cli";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api: ApiConfig,
    
    #[serde(default)]
    pub auth: AuthConfig,
    
    #[serde(default)]
    pub output: OutputConfig,
    
    #[serde(default)]
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_host")]
    pub host: String,
    
    #[serde(default = "default_port")]
    pub port: u16,
    
    #[serde(default = "default_protocol")]
    pub protocol: String,
    
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    
    #[serde(default = "default_token_expiry")]
    pub token_expiry_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,
    
    #[serde(default = "default_color")]
    pub color: bool,
    
    #[serde(default = "default_pager")]
    pub pager: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default = "default_verify")]
    pub verify: bool,
    
    pub ca_cert: Option<PathBuf>,
    pub client_cert: Option<PathBuf>,
    pub client_key: Option<PathBuf>,
}

// Default value functions
fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    55000
}

fn default_protocol() -> String {
    "https".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

fn default_token_expiry() -> u32 {
    24
}

fn default_format() -> String {
    "table".to_string()
}

fn default_color() -> bool {
    true
}

fn default_pager() -> bool {
    true
}

fn default_verify() -> bool {
    true
}

// Default implementations
impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            auth: AuthConfig::default(),
            output: OutputConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            protocol: default_protocol(),
            timeout: default_timeout(),
            max_retries: default_retries(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            username: None,
            password: None,
            token: None,
            token_expiry_hours: default_token_expiry(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            color: default_color(),
            pager: default_pager(),
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify: default_verify(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load(path: &Path) -> Result<Self> {
        let config_path = if path.to_string_lossy() == "~/.wazuh-cli/config.toml" {
            Self::default_config_path()?
        } else {
            path.to_path_buf()
        };

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            
            toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let config_path = if path.to_string_lossy() == "~/.wazuh-cli/config.toml" {
            Self::default_config_path()?
        } else {
            path.to_path_buf()
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        
        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// Get default configuration path
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .context("Failed to get system config directory")?;
        
        Ok(config_dir.join(APP_NAME).join(DEFAULT_CONFIG_FILE))
    }

    /// Get API base URL
    pub fn api_url(&self) -> String {
        format!("{}://{}:{}", self.api.protocol, self.api.host, self.api.port)
    }

    /// Update authentication token
    pub fn update_token(&mut self, token: String) {
        self.auth.token = Some(token);
    }

    /// Clear authentication token
    pub fn clear_token(&mut self) {
        self.auth.token = None;
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth.token.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api.host, "localhost");
        assert_eq!(config.api.port, 55000);
        assert_eq!(config.api.protocol, "https");
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        
        let mut config = Config::default();
        config.api.host = "test.example.com".to_string();
        config.auth.username = Some("testuser".to_string());
        
        config.save(&config_path).unwrap();
        
        let loaded_config = Config::load(&config_path).unwrap();
        assert_eq!(loaded_config.api.host, "test.example.com");
        assert_eq!(loaded_config.auth.username, Some("testuser".to_string()));
    }

    #[test]
    fn test_api_url() {
        let config = Config::default();
        assert_eq!(config.api_url(), "https://localhost:55000");
    }
}