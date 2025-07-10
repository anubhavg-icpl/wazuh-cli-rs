use anyhow::Result;
use colored::Colorize;
use std::env;
use std::process::Command;

use crate::{
    cli::{ConfigAction, ConfigCommand},
    config::Config,
    output::{print_json, print_success, print_info},
};

pub async fn handle_config_command(
    cmd: ConfigCommand,
    config: &Config,
    json_output: bool,
) -> Result<()> {
    match cmd.action {
        ConfigAction::Show => show_config(config, json_output),
        ConfigAction::Set { key, value } => set_config_value(config, &key, &value),
        ConfigAction::Get { key } => get_config_value(config, &key, json_output),
        ConfigAction::Init { force } => init_config(force),
        ConfigAction::Edit => edit_config(),
    }
}

fn show_config(config: &Config, json_output: bool) -> Result<()> {
    if json_output {
        print_json(config)?;
    } else {
        println!("{}", "Current Configuration".bold().underline());
        println!();
        
        println!("{}", "API Settings:".bold());
        println!("  Host: {}", config.api.host);
        println!("  Port: {}", config.api.port);
        println!("  Protocol: {}", config.api.protocol);
        println!("  Timeout: {} seconds", config.api.timeout);
        println!("  Max Retries: {}", config.api.max_retries);
        println!();
        
        println!("{}", "Authentication:".bold());
        println!("  Username: {}", config.auth.username.as_deref().unwrap_or("(not set)"));
        println!("  Password: {}", if config.auth.password.is_some() { "***" } else { "(not set)" });
        println!("  Token: {}", if config.auth.token.is_some() { "(set)" } else { "(not set)" });
        println!("  Token Expiry: {} hours", config.auth.token_expiry_hours);
        println!();
        
        println!("{}", "Output Settings:".bold());
        println!("  Format: {}", config.output.format);
        println!("  Color: {}", config.output.color);
        println!("  Pager: {}", config.output.pager);
        println!();
        
        println!("{}", "TLS Settings:".bold());
        println!("  Verify: {}", config.tls.verify);
        println!("  CA Certificate: {}", config.tls.ca_cert.as_ref().map(|p| p.display().to_string()).unwrap_or("(not set)".to_string()));
        println!("  Client Certificate: {}", config.tls.client_cert.as_ref().map(|p| p.display().to_string()).unwrap_or("(not set)".to_string()));
        println!("  Client Key: {}", config.tls.client_key.as_ref().map(|p| p.display().to_string()).unwrap_or("(not set)".to_string()));
    }
    
    Ok(())
}

fn set_config_value(_config: &Config, key: &str, value: &str) -> Result<()> {
    print_info(&format!("Setting {} = {}", key, value));
    
    // In a real implementation, this would modify the config and save it
    // For now, we'll just show a message
    print_info("Configuration update not yet implemented");
    print_info("Please edit the configuration file manually");
    
    Ok(())
}

fn get_config_value(config: &Config, key: &str, json_output: bool) -> Result<()> {
    let value = match key {
        "api.host" => Some(config.api.host.clone()),
        "api.port" => Some(config.api.port.to_string()),
        "api.protocol" => Some(config.api.protocol.clone()),
        "api.timeout" => Some(config.api.timeout.to_string()),
        "api.max_retries" => Some(config.api.max_retries.to_string()),
        "auth.username" => config.auth.username.clone(),
        "auth.token_expiry_hours" => Some(config.auth.token_expiry_hours.to_string()),
        "output.format" => Some(config.output.format.clone()),
        "output.color" => Some(config.output.color.to_string()),
        "output.pager" => Some(config.output.pager.to_string()),
        "tls.verify" => Some(config.tls.verify.to_string()),
        _ => None,
    };
    
    if let Some(val) = value {
        if json_output {
            print_json(&serde_json::json!({ key: val }))?;
        } else {
            println!("{} = {}", key, val);
        }
    } else {
        eprintln!("{} Unknown configuration key: {}", "Error:".red().bold(), key);
    }
    
    Ok(())
}

fn init_config(force: bool) -> Result<()> {
    let config_path = Config::default_config_path()?;
    
    if config_path.exists() && !force {
        eprintln!(
            "{} Configuration file already exists at: {}",
            "Error:".red().bold(),
            config_path.display()
        );
        eprintln!("Use --force to overwrite");
        return Ok(());
    }
    
    let default_config = Config::default();
    default_config.save(&config_path)?;
    
    print_success(&format!("Configuration initialized at: {}", config_path.display()));
    print_info("Please edit the configuration file to set your Wazuh API credentials");
    
    Ok(())
}

fn edit_config() -> Result<()> {
    let config_path = Config::default_config_path()?;
    
    if !config_path.exists() {
        eprintln!(
            "{} Configuration file not found. Run 'wazuh-cli config init' first.",
            "Error:".red().bold()
        );
        return Ok(());
    }
    
    // Try to open with the default editor
    let editor = env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".to_string()
        } else {
            "nano".to_string()
        }
    });
    
    print_info(&format!("Opening configuration file with {}", editor));
    
    Command::new(&editor)
        .arg(&config_path)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to open editor: {}", e))?;
    
    Ok(())
}