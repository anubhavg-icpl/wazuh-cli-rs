use anyhow::Result;
use colored::Colorize;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{
    cli::{AgentAction, AgentCommand},
    client::WazuhClient,
    config::Config,
    models::{AddAgentRequest, AgentListResponse, AgentParams, ApiResponse},
    output::{print_agents_table, print_json, print_single_agent},
};

pub async fn handle_agent_command(
    cmd: AgentCommand,
    config: &Config,
    json_output: bool,
) -> Result<()> {
    let config = Arc::new(RwLock::new(config.clone()));
    let client = WazuhClient::new(config).await?;
    
    // Ensure we're authenticated
    client.authenticate().await?;

    match cmd.action {
        AgentAction::List {
            status,
            os,
            version,
            count,
        } => list_agents(&client, status, os, version, count, json_output).await?,
        
        AgentAction::Get { agent } => get_agent(&client, &agent, json_output).await?,
        
        AgentAction::Add { name, ip, force } => {
            add_agent(&client, name, ip, force, json_output).await?
        }
        
        AgentAction::Remove { agent, yes } => {
            remove_agent(&client, &agent, yes, json_output).await?
        }
        
        AgentAction::Restart { agent } => restart_agent(&client, &agent, json_output).await?,
        
        AgentAction::Upgrade {
            agent,
            version,
            force,
        } => upgrade_agent(&client, &agent, version, force, json_output).await?,
        
        AgentAction::Key { agent } => get_agent_key(&client, &agent, json_output).await?,
    }

    Ok(())
}

async fn list_agents(
    client: &WazuhClient,
    status: Option<String>,
    os: Option<String>,
    version: Option<String>,
    count_only: bool,
    json_output: bool,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching agents...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let mut params = AgentParams::default();
    params.status = status;
    params.os_platform = os;
    params.version = version;

    let query_string = serde_urlencoded::to_string(&params)?;
    let url = format!("/agents?{}", query_string);
    
    debug!("Fetching agents with params: {:?}", params);
    let response = client.get(&url).await?;
    let api_response: ApiResponse<AgentListResponse> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if count_only {
        println!("Total agents: {}", api_response.data.total_affected_items);
        return Ok(());
    }

    if json_output {
        print_json(&api_response.data.affected_items)?;
    } else {
        print_agents_table(&api_response.data.affected_items);
        println!(
            "\nTotal: {} agents",
            api_response.data.total_affected_items
        );
    }

    Ok(())
}

async fn get_agent(client: &WazuhClient, agent_id: &str, json_output: bool) -> Result<()> {
    info!("Fetching agent details for: {}", agent_id);
    
    let url = format!("/agents/{}", agent_id);
    let response = client.get(&url).await?;
    let api_response: ApiResponse<AgentListResponse> = 
        WazuhClient::parse_response(response).await?;
    
    if let Some(agent) = api_response.data.affected_items.first() {
        if json_output {
            print_json(agent)?;
        } else {
            print_single_agent(agent);
        }
    } else {
        eprintln!("{} Agent '{}' not found", "Error:".red().bold(), agent_id);
    }

    Ok(())
}

async fn add_agent(
    client: &WazuhClient,
    name: String,
    ip: Option<String>,
    force: bool,
    json_output: bool,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Adding new agent...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let request = AddAgentRequest {
        name: name.clone(),
        ip,
        force: Some(force),
    };

    let response = client.post("/agents", Some(request)).await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if json_output {
        print_json(&api_response)?;
    } else {
        println!(
            "{} Agent '{}' added successfully",
            "✓".green().bold(),
            name
        );
        
        if let Some(data) = api_response.data.get("id") {
            println!("Agent ID: {}", data);
        }
        if let Some(key) = api_response.data.get("key") {
            println!("Agent key: {}", key);
        }
    }

    Ok(())
}

async fn remove_agent(
    client: &WazuhClient,
    agent_id: &str,
    skip_confirm: bool,
    json_output: bool,
) -> Result<()> {
    if !skip_confirm {
        let confirm = Confirm::new()
            .with_prompt(format!("Remove agent '{}'?", agent_id))
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Removing agent...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let url = format!("/agents/{}", agent_id);
    let response = client.delete(&url).await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if json_output {
        print_json(&api_response)?;
    } else {
        println!(
            "{} Agent '{}' removed successfully",
            "✓".green().bold(),
            agent_id
        );
    }

    Ok(())
}

async fn restart_agent(
    client: &WazuhClient,
    agent_id: &str,
    json_output: bool,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Restarting agent...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let url = if agent_id.to_lowercase() == "all" {
        "/agents/restart".to_string()
    } else {
        format!("/agents/{}/restart", agent_id)
    };

    let response = client.put(&url, None::<()>).await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if json_output {
        print_json(&api_response)?;
    } else {
        if agent_id.to_lowercase() == "all" {
            println!("{} All agents restarted successfully", "✓".green().bold());
        } else {
            println!(
                "{} Agent '{}' restarted successfully",
                "✓".green().bold(),
                agent_id
            );
        }
    }

    Ok(())
}

async fn upgrade_agent(
    client: &WazuhClient,
    agent_id: &str,
    version: Option<String>,
    force: bool,
    json_output: bool,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Upgrading agent...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let mut body = serde_json::json!({});
    if let Some(v) = version {
        body["version"] = serde_json::json!(v);
    }
    if force {
        body["force"] = serde_json::json!(true);
    }

    let url = if agent_id.to_lowercase() == "all" {
        "/agents/upgrade".to_string()
    } else {
        format!("/agents/{}/upgrade", agent_id)
    };

    let response = client.put(&url, Some(body)).await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    pb.finish_and_clear();

    if json_output {
        print_json(&api_response)?;
    } else {
        if agent_id.to_lowercase() == "all" {
            println!("{} All agents upgrade initiated", "✓".green().bold());
        } else {
            println!(
                "{} Agent '{}' upgrade initiated",
                "✓".green().bold(),
                agent_id
            );
        }
    }

    Ok(())
}

async fn get_agent_key(
    client: &WazuhClient,
    agent_id: &str,
    json_output: bool,
) -> Result<()> {
    info!("Fetching key for agent: {}", agent_id);
    
    let url = format!("/agents/{}/key", agent_id);
    let response = client.get(&url).await?;
    let api_response: ApiResponse<serde_json::Value> = 
        WazuhClient::parse_response(response).await?;
    
    if json_output {
        print_json(&api_response)?;
    } else {
        if let Some(key) = api_response.data.get("key") {
            println!("Agent key for '{}': {}", agent_id, key);
        } else {
            eprintln!("{} Could not retrieve agent key", "Error:".red().bold());
        }
    }

    Ok(())
}