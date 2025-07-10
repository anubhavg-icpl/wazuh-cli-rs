use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::process;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
mod client;
mod commands;
mod config;
mod error;
mod interactive;
mod models;
mod output;
mod utils;

use cli::{Cli, Commands};
use config::Config;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("Application error: {}", e);
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = match cli.verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let filter = EnvFilter::from_default_env()
        .add_directive(format!("wazuh_cli_rs={}", log_level).parse()?);

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    info!("Wazuh CLI starting with log level: {}", log_level);

    // Load configuration
    let config = Config::load(&cli.config)?;
    info!("Configuration loaded from: {:?}", cli.config);

    // Handle version command
    if cli.version {
        println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Execute the appropriate command
    match cli.command {
        Some(Commands::Agent(agent_cmd)) => {
            commands::agent::handle_agent_command(agent_cmd, &config, cli.json).await?;
        }
        Some(Commands::Control(control_cmd)) => {
            commands::control::handle_control_command(control_cmd, &config, cli.json).await?;
        }
        Some(Commands::Config(config_cmd)) => {
            commands::config::handle_config_command(config_cmd, &config, cli.json).await?;
        }
        Some(Commands::Interactive) => {
            interactive::start_interactive_mode(&config).await?;
        }
        None => {
            // No command provided, start interactive mode
            info!("No command provided, starting interactive mode");
            interactive::start_interactive_mode(&config).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_parsing() {
        // Test that the CLI structure can be parsed
        let cli = Cli::try_parse_from(&["wazuh-cli", "--help"]);
        assert!(cli.is_err()); // --help should cause an exit
    }
}