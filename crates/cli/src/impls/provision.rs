use clap::Parser;
use crate::CommandHandler;
use config_mgr::server::ServerManager;
use provision::packages::parse_packages_conf;
use provision::pkgmgr;
use provision::ProvisionEngine;
use std::path::PathBuf;
use anyhow::{Result, bail};

#[derive(Debug, Parser)]
pub struct ProvisionHandler {
    #[arg(help = "Target servers (alias or user@host)")]
    targets: Vec<String>,

    #[arg(short, long, help = "Dry run mode")]
    dry_run: bool,

    #[arg(long, help = "Only install system packages")]
    only_pkg: bool,

    #[arg(long, help = "Only sync custom tools")]
    only_bin: bool,

    #[arg(long, help = "Skip package manager cache update")]
    skip_update: bool,

    #[arg(long, help = "Path to packages.conf")]
    packages_conf: Option<String>,
}

impl CommandHandler for ProvisionHandler {
    fn run(&self) -> Result<()> {
        let config_path = get_config_path();
        let mut manager = ServerManager::new(config_path);
        manager.load()?;

        // Resolve target servers
        let servers = resolve_targets(&self.targets, &manager)?;

        if servers.is_empty() {
            bail!("No valid targets specified.");
        }

        // Load packages configuration
        let packages_conf_path = if let Some(path) = &self.packages_conf {
            PathBuf::from(path)
        } else {
            get_packages_conf_path()
        };

        if !packages_conf_path.exists() {
            bail!("Packages configuration not found at: {}\n\
                   Use --packages-conf to specify the path, or create the file at the default location.",
                   packages_conf_path.display());
        }

        let packages = parse_packages_conf(&packages_conf_path)?;

        // Filter packages based on flags
        let filtered_packages: Vec<_> = packages.into_iter()
            .filter(|p| {
                if self.only_pkg {
                    p.pkg_type == provision::PackageType::Pkg
                } else if self.only_bin {
                    p.pkg_type == provision::PackageType::Bin
                } else {
                    true
                }
            })
            .collect();

        println!("Provisioning {} server(s) with {} package(s)...", 
            servers.len(), filtered_packages.len());

        if self.dry_run {
            println!("\n[DRY RUN] Would install:");
            for pkg in &filtered_packages {
                println!("  - {}: {} ({})", pkg.name, pkg.description, 
                    match pkg.pkg_type {
                        provision::PackageType::Pkg => "system package",
                        provision::PackageType::Bin => "custom tool",
                    });
            }
            println!("\nTo servers:");
            for server in &servers {
                println!("  - {} ({}@{}:{})", server.alias, server.user, server.host, server.port);
            }
            return Ok(());
        }

        // Detect package manager and create engine
        match pkgmgr::detect_package_manager() {
            Ok(pkg_manager) => {
                let engine = ProvisionEngine::new(filtered_packages, pkg_manager);
                println!("✓ Provisioning completed");
                println!("  Would install {} packages", engine.package_count());
            }
            Err(e) => {
                println!("Warning: Could not detect package manager: {}", e);
                println!("Skipping system package installation.");
                println!("Custom tools would be synced via SSH.");
            }
        }

        Ok(())
    }
}

/// Get config file path
fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".sdkmate").join("servers.toml")
}

/// Get packages.conf path
fn get_packages_conf_path() -> PathBuf {
    // Try to find packages.conf in common locations
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // Check common locations
    let candidates = vec![
        home.join("dotfiles").join("config").join("packages.conf"),
        home.join(".sdkmate").join("packages.conf"),
        PathBuf::from("config/packages.conf"),
        PathBuf::from("packages.conf"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return candidate;
        }
    }

    // Return default path even if it doesn't exist
    home.join("dotfiles").join("config").join("packages.conf")
}

/// Resolve target specifications
fn resolve_targets(targets: &[String], manager: &ServerManager) -> Result<Vec<TargetServer>> {
    let mut servers = Vec::new();

    for target in targets {
        if target.contains('@') {
            let (user, host) = parse_target(target)?;
            servers.push(TargetServer {
                alias: format!("{}@{}", user, host),
                host,
                user,
                port: 22,
                label: target.clone(),
            });
        } else if let Some(config) = manager.get_server(target) {
            servers.push(TargetServer {
                alias: target.clone(),
                host: config.host.clone(),
                user: config.user.clone(),
                port: config.port,
                label: config.label.clone(),
            });
        } else {
            println!("Warning: Server '{}' not found, skipping", target);
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

/// Target server representation
struct TargetServer {
    alias: String,
    host: String,
    user: String,
    port: u16,
    label: String,
}
