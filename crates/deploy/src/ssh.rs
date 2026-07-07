use anyhow::{Context, Result, bail};
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use tokio::task;

use crate::{AuthMethod, ServerConfig};

/// SSH 会话封装
pub struct SshSession {
    session: Session,
}

impl SshSession {
    /// 执行远程命令
    pub fn exec(&self, command: &str) -> Result<String> {
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;

        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            bail!("Command '{}' failed with exit code {}", command, exit_status);
        }

        Ok(output)
    }

    /// 获取底层 session（用于文件传输）
    pub fn session(&self) -> &Session {
        &self.session
    }
}

/// 连接到服务器
pub async fn connect(server: &ServerConfig) -> Result<SshSession> {
    let host = server.host.clone();
    let port = server.port;
    let user = server.user.clone();
    let auth = server.auth.clone();

    task::spawn_blocking(move || {
        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr).with_context(|| format!("Failed to connect to {}", addr))?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        // 认证
        match auth {
            AuthMethod::Password(password) => {
                session.userauth_password(&user, &password)?;
            }
            AuthMethod::KeyPath(key_path) => {
                session.userauth_pubkey_file(&user, None, std::path::Path::new(&key_path), None)?;
            }
        }

        if !session.authenticated() {
            bail!("Authentication failed for {}", host);
        }

        Ok(SshSession { session })
    })
    .await?
}

/// 执行远程命令（单次连接）
pub async fn exec_command(server: &ServerConfig, command: &str) -> Result<String> {
    let session = connect(server).await?;
    session.exec(command)
}
