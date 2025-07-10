use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde::Serialize;

use crate::models::{Agent, AgentStatus, Service, ServiceStatus};

/// Print data as JSON
pub fn print_json<T: Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

/// Print agents in a formatted table
pub fn print_agents_table(agents: &[Agent]) {
    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Name").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("IP").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Status").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Version").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("OS").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Last Keep Alive").add_attribute(comfy_table::Attribute::Bold),
        ]);

    for agent in agents {
        let status_cell = match agent.status {
            AgentStatus::Active => Cell::new(agent.status.to_string())
                .fg(Color::Green)
                .add_attribute(comfy_table::Attribute::Bold),
            AgentStatus::Disconnected => Cell::new(agent.status.to_string())
                .fg(Color::Red),
            AgentStatus::NeverConnected => Cell::new(agent.status.to_string())
                .fg(Color::Yellow),
            AgentStatus::Pending => Cell::new(agent.status.to_string())
                .fg(Color::Blue),
        };

        let os_info = agent
            .os
            .as_ref()
            .map(|os| {
                format!(
                    "{} {}",
                    os.platform.as_deref().unwrap_or("Unknown"),
                    os.version.as_deref().unwrap_or("")
                )
                .trim()
                .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let last_keep_alive = agent
            .last_keep_alive
            .map(|dt| format_datetime(&dt))
            .unwrap_or_else(|| "Never".to_string());

        table.add_row(vec![
            Cell::new(&agent.id),
            Cell::new(&agent.name),
            Cell::new(agent.ip.as_deref().unwrap_or("N/A")),
            status_cell,
            Cell::new(agent.version.as_deref().unwrap_or("N/A")),
            Cell::new(os_info),
            Cell::new(last_keep_alive),
        ]);
    }

    println!("{table}");
}

/// Print a single agent with detailed information
pub fn print_single_agent(agent: &Agent) {
    println!("{}", "Agent Information".bold().underline());
    println!();
    
    println!("{}: {}", "ID".bold(), agent.id);
    println!("{}: {}", "Name".bold(), agent.name);
    
    if let Some(ip) = &agent.ip {
        println!("{}: {}", "IP Address".bold(), ip);
    }
    
    let status_str = match agent.status {
        AgentStatus::Active => agent.status.to_string().green(),
        AgentStatus::Disconnected => agent.status.to_string().red(),
        AgentStatus::NeverConnected => agent.status.to_string().yellow(),
        AgentStatus::Pending => agent.status.to_string().blue(),
    };
    println!("{}: {}", "Status".bold(), status_str);
    
    if let Some(version) = &agent.version {
        println!("{}: {}", "Version".bold(), version);
    }
    
    if let Some(os) = &agent.os {
        println!("{}: ", "Operating System".bold());
        if let Some(platform) = &os.platform {
            println!("  {}: {}", "Platform".bold(), platform);
        }
        if let Some(version) = &os.version {
            println!("  {}: {}", "Version".bold(), version);
        }
        if let Some(name) = &os.name {
            println!("  {}: {}", "Name".bold(), name);
        }
        if let Some(arch) = &os.arch {
            println!("  {}: {}", "Architecture".bold(), arch);
        }
    }
    
    if let Some(groups) = &agent.group {
        if !groups.is_empty() {
            println!("{}: {}", "Groups".bold(), groups.join(", "));
        }
    }
    
    if let Some(node_name) = &agent.node_name {
        println!("{}: {}", "Node".bold(), node_name);
    }
    
    if let Some(manager) = &agent.manager {
        println!("{}: {}", "Manager".bold(), manager);
    }
    
    if let Some(date_add) = &agent.date_add {
        println!("{}: {}", "Added".bold(), format_datetime(date_add));
    }
    
    if let Some(last_keep_alive) = &agent.last_keep_alive {
        println!(
            "{}: {}",
            "Last Keep Alive".bold(),
            format_datetime(last_keep_alive)
        );
    }
}

/// Print services in a formatted table
pub fn print_services_table(services: &[Service]) {
    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Service").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Status").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("PID").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Version").add_attribute(comfy_table::Attribute::Bold),
        ]);

    for service in services {
        let status_cell = match service.status {
            ServiceStatus::Running => Cell::new(service.status.to_string())
                .fg(Color::Green)
                .add_attribute(comfy_table::Attribute::Bold),
            ServiceStatus::Stopped => Cell::new(service.status.to_string())
                .fg(Color::Red),
            ServiceStatus::Unknown => Cell::new(service.status.to_string())
                .fg(Color::Yellow),
        };

        table.add_row(vec![
            Cell::new(&service.name),
            status_cell,
            Cell::new(
                service
                    .pid
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
            ),
            Cell::new(service.version.as_deref().unwrap_or("N/A")),
        ]);
    }

    println!("{table}");
}

/// Format a DateTime for display
fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}


/// Print an info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AgentOs;

    #[test]
    fn test_format_datetime() {
        let dt = DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(format_datetime(&dt), "2024-01-01 12:00:00 UTC");
    }
}