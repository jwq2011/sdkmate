use anyhow::{Context, Result};
use ssh2::Session;
use std::collections::HashMap;
use std::io::Write;
use tokio::task;

/// 渲染模板（替换 {{VAR}} 占位符）
pub fn render_template(content: &str, variables: &HashMap<String, String>) -> Result<String> {
    let mut result = content.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    Ok(result)
}

/// 上传文件到远程服务器
pub async fn upload_file(session: &Session, content: &str, remote_path: &str) -> Result<()> {
    let content = content.to_string();
    let remote_path = remote_path.to_string();
    let session = session.clone();

    task::spawn_blocking(move || {
        // 确保远程目录存在
        let dir = std::path::Path::new(&remote_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/tmp".to_string());

        let mut channel = session.channel_session()?;
        channel.exec(&format!("mkdir -p {}", dir))?;
        channel.wait_close()?;

        // 使用 scp 上传文件
        let content_bytes = content.as_bytes();
        let mut remote_file =
            session.scp_send(std::path::Path::new(&remote_path), 0o644, content_bytes.len() as u64, None)?;

        remote_file.write_all(content_bytes)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        Ok(())
    })
    .await?
}

/// 使用 rsync 传输文件（fallback: scp）
pub async fn rsync_transfer(local_path: &str, remote_path: &str, server: &super::ServerConfig) -> Result<()> {
    let local_path = local_path.to_string();
    let remote_path = remote_path.to_string();
    let host = server.host.clone();
    let user = server.user.clone();
    let port = server.port;

    task::spawn_blocking(move || {
        let remote_addr = format!("{}@{}:{}", user, host, remote_path);

        let output = std::process::Command::new("rsync")
            .args(["-avz", "-e", &format!("ssh -p {}", port), &local_path, &remote_addr])
            .output()
            .context("Failed to execute rsync")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("rsync failed: {}", stderr);
        }

        Ok(())
    })
    .await?
}
