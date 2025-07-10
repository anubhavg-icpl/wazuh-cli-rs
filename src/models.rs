use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub error: i32,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    pub status: AgentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<AgentOs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_keep_alive: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_add: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active,
    Disconnected,
    #[serde(rename = "never_connected")]
    NeverConnected,
    Pending,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Active => write!(f, "Active"),
            AgentStatus::Disconnected => write!(f, "Disconnected"),
            AgentStatus::NeverConnected => write!(f, "Never Connected"),
            AgentStatus::Pending => write!(f, "Pending"),
        }
    }
}

/// Agent operating system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOs {
    pub platform: Option<String>,
    pub version: Option<String>,
    pub name: Option<String>,
    pub arch: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub codename: Option<String>,
}

/// Agent list response
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentListResponse {
    pub affected_items: Vec<Agent>,
    pub total_affected_items: u32,
    pub total_failed_items: u32,
    pub failed_items: Vec<serde_json::Value>,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub status: ServiceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    Stopped,
    Unknown,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Running => write!(f, "Running"),
            ServiceStatus::Stopped => write!(f, "Stopped"),
            ServiceStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Manager information
#[derive(Debug, Serialize, Deserialize)]
pub struct ManagerInfo {
    pub compilation_date: Option<String>,
    pub version: String,
    pub openssl_support: bool,
    pub max_agents: u32,
    pub tz_offset: String,
    pub tz_name: String,
    pub name: String,
    pub cluster: ClusterInfo,
}

/// Cluster information
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,
}

/// Agent key information
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentKey {
    pub id: String,
    pub key: String,
}

/// Configuration item
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    pub section: String,
    pub key: String,
    pub value: serde_json::Value,
}

/// Statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub alerts: HashMap<String, u64>,
    pub events: HashMap<String, u64>,
    pub syscheck: HashMap<String, u64>,
    pub syscollector: HashMap<String, u64>,
}

/// Request parameters for agent operations
#[derive(Debug, Serialize)]
pub struct AgentParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_platform: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
}

impl Default for AgentParams {
    fn default() -> Self {
        Self {
            limit: Some(500),
            offset: None,
            sort: None,
            search: None,
            status: None,
            q: None,
            os_platform: None,
            os_version: None,
            manager: None,
            version: None,
            group: None,
            node_name: None,
        }
    }
}

/// Request body for adding a new agent
#[derive(Debug, Serialize)]
pub struct AddAgentRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// Response from adding a new agent
#[derive(Debug, Deserialize)]
pub struct AddAgentResponse {
    pub id: String,
    pub key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_display() {
        assert_eq!(AgentStatus::Active.to_string(), "Active");
        assert_eq!(AgentStatus::Disconnected.to_string(), "Disconnected");
        assert_eq!(AgentStatus::NeverConnected.to_string(), "Never Connected");
        assert_eq!(AgentStatus::Pending.to_string(), "Pending");
    }

    #[test]
    fn test_service_status_display() {
        assert_eq!(ServiceStatus::Running.to_string(), "Running");
        assert_eq!(ServiceStatus::Stopped.to_string(), "Stopped");
        assert_eq!(ServiceStatus::Unknown.to_string(), "Unknown");
    }
}