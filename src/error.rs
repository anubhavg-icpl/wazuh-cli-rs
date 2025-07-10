use thiserror::Error;

#[derive(Error, Debug)]
pub enum WazuhError {
    #[error("API error ({code}): {message}")]
    ApiError { code: i32, message: String },

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for WazuhError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            WazuhError::Timeout
        } else if err.is_connect() {
            WazuhError::NetworkError(format!("Connection failed: {}", err))
        } else {
            WazuhError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for WazuhError {
    fn from(err: serde_json::Error) -> Self {
        WazuhError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for WazuhError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => WazuhError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => WazuhError::PermissionDenied(err.to_string()),
            _ => WazuhError::Unknown(err.to_string()),
        }
    }
}

/// Result type alias for Wazuh operations
pub type WazuhResult<T> = Result<T, WazuhError>;