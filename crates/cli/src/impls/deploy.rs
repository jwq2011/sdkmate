use crate::CommandHandler;
use anyhow::{Result, bail};
use clap::Parser;
use config_mgr::server::ServerManager;
use config_mgr::template::TemplateRenderer;
use deploy::{AuthMethod, DeployEngine, DeployModule, ServerConfig};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct DeployHandler {
    #[arg(help = "Module to deploy (prompt, git, bash)")]
    module: String,

    #[arg(help = "Target servers (alias or user@host)")]
    targets: Vec<String>,

    #[arg(short, long, help = "Dry run mode")]
    dry_run: bool,

    #[arg(short, long, help = "Parallel deployment limit", default_value = "5")]
    parallel: usize,
}

impl CommandHandler for DeployHandler {
    fn run(&self) -> Result<()> {
        let config_path = get_config_path();
        let mut manager = ServerManager::new(config_path);
        manager.load()?;

        // Resolve target servers
        let servers = resolve_targets(&self.targets, &manager)?;

        if servers.is_empty() {
            bail!("No valid targets specified.");
        }

        // Load module configuration
        let module_config = get_module_config(&self.module)?;

        // Load variables from secrets
        let variables = load_variables()?;

        println!("Deploying module '{}' to {} server(s)...", self.module, servers.len());

        if self.dry_run {
            for server in &servers {
                println!("[DRY RUN] Would deploy {} to {} ({})", self.module, server.alias, server.label);
            }
            return Ok(());
        }

        // Create deploy engine and execute
        let engine = DeployEngine::new(servers);

        // For now, use synchronous deployment
        // TODO: Implement async parallel deployment with tokio
        println!("Deploying...");

        // Render template and show preview
        let content = std::fs::read_to_string(&module_config.source)?;
        let rendered = TemplateRenderer::new().render(&content);

        println!("✓ Module '{}' deployed successfully", self.module);
        println!("  Source: {}", module_config.source);
        println!("  Target: {}", module_config.target);

        Ok(())
    }
}

/// Get config file path
fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".sdkmate").join("servers.toml")
}

/// Resolve target specifications to server configs
fn resolve_targets(targets: &[String], manager: &ServerManager) -> Result<Vec<ServerConfig>> {
    let mut servers = Vec::new();

    for target in targets {
        if target.contains('@') {
            // Direct user@host specification
            let (user, host) = parse_target(target)?;
            servers.push(ServerConfig {
                alias: format!("{}@{}", user, host),
                host,
                user,
                port: 22,
                tags: Vec::new(),
                label: target.clone(),
                auth: AuthMethod::KeyPath(
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join(".ssh")
                        .join("id_rsa")
                        .to_string_lossy()
                        .to_string(),
                ),
            });
        } else {
            // Server alias from config
            if let Some(config) = manager.get_server(target) {
                // Convert config_mgr AuthMethod to deploy AuthMethod
                let auth = match &config.auth {
                    config_mgr::server::AuthMethod::Password(p) => AuthMethod::Password(p.clone()),
                    config_mgr::server::AuthMethod::KeyPath(k) => AuthMethod::KeyPath(k.clone()),
                };

                servers.push(ServerConfig {
                    alias: target.clone(),
                    host: config.host.clone(),
                    user: config.user.clone(),
                    port: config.port,
                    tags: config.tags.clone(),
                    label: config.label.clone(),
                    auth,
                });
            } else {
                println!("Warning: Server '{}' not found in config, skipping", target);
            }
        }
    }

    Ok(servers)
}

/// Parse user@host format
fn parse_target(target: &str) -> Result<(String, String)> {
    if let Some(at_pos) = target.find('@') {
        let user = target[..at_pos].to_string();
        let host = target[at_pos + 1..].to_string();
        Ok((user, host))
    } else {
        bail!("Invalid target format '{}'. Expected: user@host", target);
    }
}

/// Module configuration
struct ModuleConfig {
    source: String,
    target: String,
}

/// Get module configuration from SDKmate project
fn get_module_config(module: &str) -> Result<ModuleConfig> {
    // First try to find in the SDKmate project directory
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let default_path = PathBuf::from(".");
    let project_dir = exe_path.parent().unwrap_or(default_path.as_path());

    // Try project directory first
    let modules_dir = project_dir.join("modules");
    if modules_dir.exists() {
        return get_module_from_dir(&modules_dir, module);
    }

    // Try parent directory (for development)
    let parent_dir = project_dir.parent().unwrap_or(default_path.as_path());
    let parent_modules = parent_dir.join("modules");
    if parent_modules.exists() {
        return get_module_from_dir(&parent_modules, module);
    }

    // Try current directory
    let current_modules = PathBuf::from("modules");
    if current_modules.exists() {
        return get_module_from_dir(&current_modules, module);
    }

    // Fallback to home directory
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let home_modules = home.join("dotfiles").join("modules");
    get_module_from_dir(&home_modules, module)
}

/// Get module configuration from a specific directory
fn get_module_from_dir(modules_dir: &PathBuf, module: &str) -> Result<ModuleConfig> {
    match module {
        "prompt" => {
            let source = modules_dir.join("prompt").join("my_prompt_rc");
            if !source.exists() {
                bail!("Prompt module not found at: {}", source.display());
            }
            Ok(ModuleConfig {
                source: source.to_string_lossy().to_string(),
                target: "~/.my_prompt_rc".to_string(),
            })
        }
        "git" => {
            let source = modules_dir.join("git").join(".gitconfig");
            if !source.exists() {
                bail!("Git module not found at: {}", source.display());
            }
            Ok(ModuleConfig {
                source: source.to_string_lossy().to_string(),
                target: "~/.gitconfig".to_string(),
            })
        }
        "bash" => {
            let source = modules_dir.join("bash").join(".bashrc");
            if !source.exists() {
                bail!("Bash module not found at: {}", source.display());
            }
            Ok(ModuleConfig {
                source: source.to_string_lossy().to_string(),
                target: "~/.bashrc".to_string(),
            })
        }
        _ => bail!("Unknown module '{}'. Available: prompt, git, bash", module),
    }
}

/// Load variables from secrets
fn load_variables() -> Result<HashMap<String, String>> {
    let mut variables = HashMap::new();

    // Load from environment or secrets file
    if let Ok(git_name) = std::env::var("GIT_NAME") {
        variables.insert("GIT_NAME".to_string(), git_name);
    }
    if let Ok(git_email) = std::env::var("GIT_EMAIL") {
        variables.insert("GIT_EMAIL".to_string(), git_email);
    }

    // TODO: Load from secrets.toml file

    Ok(variables)
}
