use clap::Parser;
use crate::CommandHandler;
use config_mgr::server::{ServerManager, ServerConfig, AuthMethod};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Parser)]
pub struct ServerHandler {
    #[command(subcommand)]
    command: ServerCommands,
}

#[derive(Debug, Parser)]
pub enum ServerCommands {
    #[command(name = "add", about = "Add a new server")]
    Add {
        #[arg(help = "Server alias")]
        alias: String,
        #[arg(help = "User@Host (e.g., root@192.168.1.100)")]
        target: String,
        #[arg(short, long, help = "SSH port", default_value = "22")]
        port: u16,
        #[arg(short, long, help = "Server label")]
        label: Option<String>,
        #[arg(long, help = "SSH key path")]
        key: Option<String>,
        #[arg(long, help = "Tags (comma-separated)")]
        tags: Option<String>,
    },

    #[command(name = "remove", about = "Remove a server")]
    Remove {
        #[arg(help = "Server alias")]
        alias: String,
    },

    #[command(name = "list", about = "List all servers")]
    List {
        #[arg(short, long, help = "Filter by tag")]
        filter: Option<String>,
    },

    #[command(name = "status", about = "Show server status")]
    Status {
        #[arg(short, long, help = "Filter by tag")]
        filter: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
}

impl CommandHandler for ServerHandler {
    fn run(&self) -> Result<()> {
        let config_path = get_config_path();
        let mut manager = ServerManager::new(config_path);
        manager.load()?;

        match &self.command {
            ServerCommands::Add { alias, target, port, label, key, tags } => {
                let (user, host) = parse_target(target)?;

                let auth = if let Some(key_path) = key {
                    AuthMethod::KeyPath(key_path.clone())
                } else {
                    // Default to key-based auth
                    let default_key = dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join(".ssh")
                        .join("id_rsa");
                    AuthMethod::KeyPath(default_key.to_string_lossy().to_string())
                };

                let tags_vec = tags
                    .as_ref()
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();

                let label_str = label.clone().unwrap_or_else(|| alias.clone());

                let server_config = ServerConfig {
                    host: host.clone(),
                    user: user.clone(),
                    port: *port,
                    tags: tags_vec,
                    label: label_str,
                    auth,
                };

                manager.add_server(alias.clone(), server_config)?;
                println!("✓ Added server: {} ({}@{}:{})", alias, user, host, port);
                Ok(())
            }
            ServerCommands::Remove { alias } => {
                manager.remove_server(alias)?;
                println!("✓ Removed server: {}", alias);
                Ok(())
            }
            ServerCommands::List { filter } => {
                let servers: Vec<_> = if let Some(tag) = filter {
                    manager.filter_by_tag(tag)
                        .iter()
                        .map(|(alias, config)| (alias.to_string(), (*config).clone()))
                        .collect()
                } else {
                    manager.list_servers()
                        .iter()
                        .map(|(alias, config)| (alias.clone(), config.clone()))
                        .collect()
                };

                if servers.is_empty() {
                    println!("No servers configured.");
                    println!("Use 'sdkm server add <alias> <user@host>' to add a server.");
                } else {
                    println!("{:<15} {:<25} {:<8} {:<20} TAGS", 
                        "ALIAS", "HOST", "PORT", "LABEL");
                    println!("{}", "-".repeat(80));
                    for (alias, config) in &servers {
                        println!("{:<15} {:<25} {:<8} {:<20} {}",
                            alias,
                            format!("{}@{}", config.user, config.host),
                            config.port,
                            config.label,
                            config.tags.join(", "));
                    }
                }
                Ok(())
            }
            ServerCommands::Status { filter, json: _ } => {
                // TODO: Implement actual SSH status check
                println!("Checking server status...");
                let servers: Vec<_> = if let Some(tag) = filter {
                    manager.filter_by_tag(tag)
                        .iter()
                        .map(|(alias, config)| (alias.to_string(), (*config).clone()))
                        .collect()
                } else {
                    manager.list_servers()
                        .iter()
                        .map(|(alias, config)| (alias.clone(), config.clone()))
                        .collect()
                };

                if servers.is_empty() {
                    println!("No servers to check.");
                } else {
                    println!("Would check {} servers (SSH status check not yet implemented)", servers.len());
                }
                Ok(())
            }
        }
    }
}

/// Get config file path
fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".sdkmate").join("servers.toml")
}

/// Parse user@host format
fn parse_target(target: &str) -> Result<(String, String)> {
    if let Some(at_pos) = target.find('@') {
        let user = target[..at_pos].to_string();
        let host = target[at_pos + 1..].to_string();
        Ok((user, host))
    } else {
        anyhow::bail!("Invalid target format '{}'. Expected: user@host", target);
    }
}
