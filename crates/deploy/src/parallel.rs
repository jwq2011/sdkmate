use std::collections::HashMap;
use tokio::sync::Semaphore;

use crate::ssh;
use crate::transfer;
use crate::{DeployModule, DeployResult, ServerConfig};

/// 并行部署到多台服务器
pub async fn deploy_parallel(
    servers: &[ServerConfig],
    module: &DeployModule,
    variables: &HashMap<String, String>,
    parallel_limit: usize,
) -> Vec<DeployResult> {
    let semaphore = std::sync::Arc::new(Semaphore::new(parallel_limit));
    let mut handles = Vec::new();

    for server in servers {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let server = server.clone();
        let module = module.clone();
        let variables = variables.clone();

        let handle = tokio::spawn(async move {
            let result = deploy_single(&server, &module, &variables).await;
            drop(permit);
            result
        });

        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(DeployResult {
                server: "unknown".to_string(),
                success: false,
                message: format!("Task panicked: {}", e),
            }),
        }
    }

    results
}

/// 部署到单台服务器
async fn deploy_single(
    server: &ServerConfig,
    module: &DeployModule,
    variables: &HashMap<String, String>,
) -> DeployResult {
    match deploy_to_server(server, module, variables).await {
        Ok(result) => result,
        Err(e) => DeployResult {
            server: server.alias.clone(),
            success: false,
            message: format!("Deploy failed: {}", e),
        },
    }
}

/// 部署到单台服务器的内部实现
async fn deploy_to_server(
    server: &ServerConfig,
    module: &DeployModule,
    variables: &HashMap<String, String>,
) -> anyhow::Result<DeployResult> {
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
