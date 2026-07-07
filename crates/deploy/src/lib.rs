pub mod parallel;
pub mod ssh;
pub mod transfer;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub alias: String,
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

/// 部署模块定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployModule {
    pub name: String,
    pub source: String,
    pub target: String,
    pub placeholders: Vec<String>,
}

/// 部署结果
#[derive(Debug)]
pub struct DeployResult {
    pub server: String,
    pub success: bool,
    pub message: String,
}

/// 部署引擎
pub struct DeployEngine {
    servers: Vec<ServerConfig>,
}

impl DeployEngine {
    pub fn new(servers: Vec<ServerConfig>) -> Self {
        Self { servers }
    }

    /// 部署到单台服务器
    pub async fn deploy_to_server(
        &self,
        server: &ServerConfig,
        module: &DeployModule,
        variables: &HashMap<String, String>,
    ) -> Result<DeployResult> {
        let session = ssh::connect(server).await?;

        // 渲染模板
        let content = std::fs::read_to_string(&module.source)?;
        let rendered = transfer::render_template(&content, variables)?;

        // 上传文件
        transfer::upload_file(session.session(), &rendered, &module.target).await?;

        Ok(DeployResult {
            server: server.alias.clone(),
            success: true,
            message: format!("Deployed {} to {}", module.name, server.alias),
        })
    }

    /// 并行部署到多台服务器
    pub async fn deploy_parallel(
        &self,
        module: &DeployModule,
        variables: &HashMap<String, String>,
        parallel_limit: usize,
    ) -> Vec<DeployResult> {
        parallel::deploy_parallel(&self.servers, module, variables, parallel_limit).await
    }
}
