use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    cli::{ControlAction, ControlCommand},
    client::WazuhClient,
    config::Config,
    models::{ApiResponse, Service},
    output::{print_json, print_services_table, print_success},
};

pub async fn handle_control_command(
    cmd: ControlCommand,
    config: &Config,
    json_output: bool,
) -> Result<()> {
    let config = Arc::new(RwLock::new(config.clone()));
    let client = WazuhClient::new(config).await?;
    
    // Ensure we're authenticated
    client.authenticate().await?;

    match cmd.action {
        ControlAction::Status { service } => {
            get_service_status(&client, service, json_output).await?
        }
        ControlAction::Start { service } => {
            start_service(&client, service, json_output).await?
        }
        ControlAction::Stop { service } => {
            stop_service(&client, service, json_output).await?
        }
        ControlAction::Restart { service } => {
            restart_service(&client, service, json_output).await?
        }
        ControlAction::Info => get_manager_info(&client, json_output).await?,
    }

    Ok(())
}

async fn get_service_status(
    client: &WazuhClient,
    service: Option<String>,
    json_output: bool,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching service status...");
    pb.enable_steady_tick(Duration::from_millis(120));

    // Get manager status which includes service information
    let response = client.get("/manager/status").await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    // Parse services from the response
    let services = parse_services_from_status(&api_response.data)?;

    if let Some(service_name) = service {
        // Filter to specific service
        let filtered: Vec<Service> = services
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&service_name.to_lowercase()))
            .collect();

        if filtered.is_empty() {
            eprintln!(
                "{} Service '{}' not found",
                "Error:".red().bold(),
                service_name
            );
            return Ok(());
        }

        if json_output {
            print_json(&filtered)?;
        } else {
            print_services_table(&filtered);
        }
    } else {
        // Show all services
        if json_output {
            print_json(&services)?;
        } else {
            println!("{}", "Wazuh Services Status".bold().underline());
            println!();
            print_services_table(&services);
        }
    }

    Ok(())
}

async fn start_service(
    client: &WazuhClient,
    service: Option<String>,
    json_output: bool,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    
    let service_name = service.as_deref().unwrap_or("all");
    pb.set_message(format!("Starting {}...", service_name));
    pb.enable_steady_tick(Duration::from_millis(120));

    let url = if service.is_some() && service.as_ref().unwrap() != "all" {
        format!("/manager/restart?service={}", service.as_ref().unwrap())
    } else {
        "/manager/restart".to_string()
    };

    let response = client.put(&url, None::<()>).await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if json_output {
        print_json(&api_response)?;
    } else {
        if service_name == "all" {
            print_success("All services started successfully");
        } else {
            print_success(&format!("Service '{}' started successfully", service_name));
        }
    }

    Ok(())
}

async fn stop_service(
    _client: &WazuhClient,
    service: Option<String>,
    _json_output: bool,
) -> Result<()> {
    if service.is_none() || service.as_ref().unwrap() == "all" {
        eprintln!(
            "{} Cannot stop all services. Please specify a service name.",
            "Error:".red().bold()
        );
        return Ok(());
    }

    let _service_name = service.unwrap();
    
    // Note: Wazuh API doesn't have a direct stop endpoint for individual services
    // This is a limitation of the current API
    eprintln!(
        "{} Stopping individual services is not supported by the Wazuh API.",
        "Warning:".yellow().bold()
    );
    eprintln!("Please use system service management commands (systemctl, service) to stop services.");
    
    Ok(())
}

async fn restart_service(
    client: &WazuhClient,
    service: Option<String>,
    json_output: bool,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    
    let service_name = service.as_deref().unwrap_or("all");
    pb.set_message(format!("Restarting {}...", service_name));
    pb.enable_steady_tick(Duration::from_millis(120));

    let url = "/manager/restart".to_string();

    let response = client.put(&url, None::<()>).await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if json_output {
        print_json(&api_response)?;
    } else {
        if service_name == "all" {
            print_success("All services restarted successfully");
        } else {
            print_success(&format!("Manager restart initiated (affects all services)"));
        }
    }

    Ok(())
}

async fn get_manager_info(client: &WazuhClient, json_output: bool) -> Result<()> {
    info!("Fetching manager information");
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching manager info...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let response = client.get("/manager/info").await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if json_output {
        print_json(&api_response.data)?;
    } else {
        print_manager_info(&api_response.data)?;
    }

    Ok(())
}

fn parse_services_from_status(data: &serde_json::Value) -> Result<Vec<Service>> {
    let mut services = Vec::new();

    // The status response structure varies, but typically includes service status
    if let Some(obj) = data.as_object() {
        for (name, status) in obj {
            let service = Service {
                name: name.clone(),
                status: parse_service_status(status),
                pid: None,
                version: None,
            };
            services.push(service);
        }
    }

    Ok(services)
}

fn parse_service_status(value: &serde_json::Value) -> crate::models::ServiceStatus {
    if let Some(status_str) = value.as_str() {
        match status_str.to_lowercase().as_str() {
            "running" => crate::models::ServiceStatus::Running,
            "stopped" => crate::models::ServiceStatus::Stopped,
            _ => crate::models::ServiceStatus::Unknown,
        }
    } else {
        crate::models::ServiceStatus::Unknown
    }
}

fn print_manager_info(data: &serde_json::Value) -> Result<()> {
    println!("{}", "Wazuh Manager Information".bold().underline());
    println!();

    if let Some(version) = data.get("version") {
        println!("{}: {}", "Version".bold(), version);
    }

    if let Some(name) = data.get("name") {
        println!("{}: {}", "Name".bold(), name);
    }

    if let Some(compilation_date) = data.get("compilation_date") {
        println!("{}: {}", "Compilation Date".bold(), compilation_date);
    }

    if let Some(max_agents) = data.get("max_agents") {
        println!("{}: {}", "Max Agents".bold(), max_agents);
    }

    if let Some(openssl) = data.get("openssl_support") {
        println!("{}: {}", "OpenSSL Support".bold(), openssl);
    }

    if let Some(tz_offset) = data.get("tz_offset") {
        println!("{}: {}", "Timezone Offset".bold(), tz_offset);
    }

    if let Some(tz_name) = data.get("tz_name") {
        println!("{}: {}", "Timezone".bold(), tz_name);
    }

    if let Some(cluster) = data.get("cluster") {
        println!();
        println!("{}", "Cluster Information".bold());
        
        if let Some(enabled) = cluster.get("enabled") {
            println!("  {}: {}", "Enabled".bold(), enabled);
        }
        
        if let Some(node_name) = cluster.get("node_name") {
            println!("  {}: {}", "Node Name".bold(), node_name);
        }
        
        if let Some(node_type) = cluster.get("node_type") {
            println!("  {}: {}", "Node Type".bold(), node_type);
        }
    }

    Ok(())
}