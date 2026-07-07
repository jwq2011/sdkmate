pub mod secrets;
pub mod server;
pub mod template;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevEnvConfig {
    pub global: GlobalConfig,
    pub servers: HashMap<String, server::ServerConfig>,
    pub deploy: DeployConfig,
    pub provision: ProvisionConfig,
}

/// 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub default_user: String,
    pub default_port: u16,
}

/// 部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub default_module: String,
    pub parallel_limit: usize,
    pub modules: HashMap<String, DeployModuleConfig>,
}

/// 部署模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployModuleConfig {
    pub source: String,
    pub target: String,
    pub placeholders: Vec<String>,
}

/// 预配配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionConfig {
    pub packages_conf: String,
    pub bin_manifest: String,
    pub bin_dir: String,
}

impl DevEnvConfig {
    /// 从文件加载配置
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: DevEnvConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 初始化默认配置
    pub fn default_config() -> Self {
        Self {
            global: GlobalConfig {
                default_user: "root".to_string(),
                default_port: 22,
            },
            servers: HashMap::new(),
            deploy: DeployConfig {
                default_module: "prompt".to_string(),
                parallel_limit: 5,
                modules: HashMap::new(),
            },
            provision: ProvisionConfig {
                packages_conf: "config/packages.conf".to_string(),
                bin_manifest: "config/bin-manifest.json".to_string(),
                bin_dir: "bin/".to_string(),
            },
        }
    }
}
