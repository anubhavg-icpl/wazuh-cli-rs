use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "wazuh-cli",
    author = "Security Engineer",
    version,
    about = "Modern CLI for Wazuh SIEM management",
    long_about = "A powerful command-line interface for managing Wazuh security platform.\n\
                  Supports agent management, service control, and configuration."
)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE", default_value = "~/.wazuh-cli/config.toml")]
    pub config: PathBuf,

    /// Output format (json or table)
    #[arg(short, long, default_value = "table")]
    pub output: String,

    /// Enable JSON output
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Verbosity level (can be repeated)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Show version information
    #[arg(short = 'V', long)]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage Wazuh agents
    #[command(aliases = &["agents", "a"])]
    Agent(AgentCommand),

    /// Control Wazuh services
    #[command(aliases = &["ctl", "c"])]
    Control(ControlCommand),

    /// Manage configuration
    #[command(aliases = &["cfg"])]
    Config(ConfigCommand),

    /// Start interactive mode
    #[command(aliases = &["i", "shell"])]
    Interactive,
}

#[derive(Parser)]
pub struct AgentCommand {
    #[command(subcommand)]
    pub action: AgentAction,
}

#[derive(Subcommand)]
pub enum AgentAction {
    /// List all agents
    #[command(aliases = &["ls", "l"])]
    List {
        /// Filter by status (active, disconnected, never_connected, pending)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by operating system
        #[arg(short, long)]
        os: Option<String>,

        /// Filter by version
        #[arg(short = 'v', long)]
        version: Option<String>,

        /// Show only agent count
        #[arg(short, long)]
        count: bool,
    },

    /// Show agent details
    #[command(aliases = &["info", "show", "i"])]
    Get {
        /// Agent ID or name
        agent: String,
    },

    /// Add a new agent
    #[command(aliases = &["create", "new"])]
    Add {
        /// Agent name
        #[arg(short, long)]
        name: String,

        /// Agent IP address
        #[arg(short, long)]
        ip: Option<String>,

        /// Force agent creation
        #[arg(short, long)]
        force: bool,
    },

    /// Remove an agent
    #[command(aliases = &["rm", "del", "delete"])]
    Remove {
        /// Agent ID or name
        agent: String,

        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Restart an agent
    Restart {
        /// Agent ID, name, or "all"
        agent: String,
    },

    /// Upgrade agent
    Upgrade {
        /// Agent ID, name, or "all"
        agent: String,

        /// Target version
        #[arg(short, long)]
        version: Option<String>,

        /// Force upgrade
        #[arg(short, long)]
        force: bool,
    },

    /// Get agent key
    Key {
        /// Agent ID or name
        agent: String,
    },
}

#[derive(Parser)]
pub struct ControlCommand {
    #[command(subcommand)]
    pub action: ControlAction,
}

#[derive(Subcommand)]
pub enum ControlAction {
    /// Show service status
    Status {
        /// Service name (optional)
        service: Option<String>,
    },

    /// Start services
    Start {
        /// Service name or "all"
        service: Option<String>,
    },

    /// Stop services
    Stop {
        /// Service name or "all"
        service: Option<String>,
    },

    /// Restart services
    Restart {
        /// Service name or "all"
        service: Option<String>,
    },

    /// Show service information
    Info,
}

#[derive(Parser)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Initialize configuration
    Init {
        /// Force initialization
        #[arg(short, long)]
        force: bool,
    },

    /// Edit configuration in editor
    Edit,
}