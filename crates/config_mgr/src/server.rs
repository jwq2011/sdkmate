use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub tags: Vec<String>,
    pub label: String,
    /// 认证方式：password 或 key_path
    pub auth: AuthMethod,
}

/// 认证方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password(String),
    KeyPath(String),
}

/// 服务器配置管理器
pub struct ServerManager {
    config_path: PathBuf,
    servers: HashMap<String, ServerConfig>,
}

impl ServerManager {
    /// 创建新的服务器管理器
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            servers: HashMap::new(),
        }
    }

    /// 从配置文件加载服务器列表
    pub fn load(&mut self) -> Result<()> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.config_path).context("Failed to read server config")?;

        let config: toml::Value = toml::from_str(&content).context("Failed to parse server config")?;

        if let Some(servers) = config.get("servers").and_then(|s| s.as_table()) {
            for (alias, value) in servers {
                let server: ServerConfig = value
                    .clone()
                    .try_into()
                    .context(format!("Failed to parse server '{}'", alias))?;
                self.servers.insert(alias.clone(), server);
            }
        }

        Ok(())
    }

    /// 保存服务器配置到文件
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let mut config = toml::Value::Table(toml::map::Map::new());

        let mut servers_table = toml::map::Map::new();
        for (alias, server) in &self.servers {
            let server_value =
                toml::Value::try_from(server).context(format!("Failed to serialize server '{}'", alias))?;
            servers_table.insert(alias.clone(), server_value);
        }

        config
            .as_table_mut()
            .unwrap()
            .insert("servers".to_string(), toml::Value::Table(servers_table));

        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&self.config_path, content)?;

        Ok(())
    }

    /// 添加服务器
    pub fn add_server(&mut self, alias: String, config: ServerConfig) -> Result<()> {
        if self.servers.contains_key(&alias) {
            bail!("Server '{}' already exists", alias);
        }
        self.servers.insert(alias, config);
        self.save()
    }

    /// 移除服务器
    pub fn remove_server(&mut self, alias: &str) -> Result<()> {
        if !self.servers.contains_key(alias) {
            bail!("Server '{}' not found", alias);
        }
        self.servers.remove(alias);
        self.save()
    }

    /// 获取服务器配置
    pub fn get_server(&self, alias: &str) -> Option<&ServerConfig> {
        self.servers.get(alias)
    }

    /// 获取所有服务器
    pub fn list_servers(&self) -> &HashMap<String, ServerConfig> {
        &self.servers
    }

    /// 按标签过滤服务器
    pub fn filter_by_tag(&self, tag: &str) -> Vec<(&String, &ServerConfig)> {
        self.servers
            .iter()
            .filter(|(_, server)| server.tags.contains(&tag.to_string()))
            .collect()
    }
}
