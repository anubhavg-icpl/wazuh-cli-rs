use anyhow::Result;
use colored::Colorize;
use dialoguer::Input;
use std::io::{self, Write};

use crate::config::Config;
use crate::output::print_info;

pub async fn start_interactive_mode(_config: &Config) -> Result<()> {
    println!("{}", "Wazuh CLI - Interactive Mode".bold().blue());
    println!("Type 'help' for commands, 'exit' to quit\n");

    loop {
        // Show prompt
        print!("{} ", "wazuh>".green().bold());
        io::stdout().flush()?;

        // Read input
        let input = Input::<String>::new()
            .allow_empty(true)
            .interact_text()?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Parse command
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0] {
            "help" | "?" => show_help(),
            "exit" | "quit" | "q" => {
                println!("Goodbye!");
                break;
            }
            "agents" => {
                print_info("Agent management commands - not yet implemented in interactive mode");
            }
            "control" => {
                print_info("Service control commands - not yet implemented in interactive mode");
            }
            "config" => {
                print_info("Configuration commands - not yet implemented in interactive mode");
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
            }
            _ => {
                eprintln!(
                    "{} Unknown command: '{}'. Type 'help' for available commands.",
                    "Error:".red().bold(),
                    parts[0]
                );
            }
        }
    }

    Ok(())
}

fn show_help() {
    println!("{}", "Available Commands:".bold().underline());
    println!();
    println!("  {}  - Show this help message", "help".green());
    println!("  {} - List and manage agents", "agents".green());
    println!("  {} - Control Wazuh services", "control".green());
    println!("  {} - Manage configuration", "config".green());
    println!("  {}  - Clear the screen", "clear".green());
    println!("  {}  - Exit interactive mode", "exit".green());
    println!();
    println!("For detailed command help, use: <command> --help");
}