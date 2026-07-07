use anyhow::Result;
use async_trait::async_trait;

/// 包管理器 trait
#[async_trait]
pub trait PackageManager: Send + Sync {
    /// 安装包
    async fn install(&self, package: &str) -> Result<()>;

    /// 卸载包
    async fn uninstall(&self, package: &str) -> Result<()>;

    /// 检查包是否已安装
    async fn is_installed(&self, package: &str) -> Result<bool>;

    /// 更新包列表
    async fn update(&self) -> Result<()>;

    /// 获取包管理器名称
    fn name(&self) -> &str;
}

/// apt 包管理器（Debian/Ubuntu）
pub struct AptManager;

#[async_trait]
impl PackageManager for AptManager {
    async fn install(&self, package: &str) -> Result<()> {
        let output = tokio::process::Command::new("apt-get")
            .args(["install", "-y", package])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("apt-get install failed: {}", stderr);
        }

        Ok(())
    }

    async fn uninstall(&self, package: &str) -> Result<()> {
        let output = tokio::process::Command::new("apt-get")
            .args(["remove", "-y", package])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("apt-get remove failed: {}", stderr);
        }

        Ok(())
    }

    async fn is_installed(&self, package: &str) -> Result<bool> {
        let output = tokio::process::Command::new("dpkg").args(["-l", package]).output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(&format!("ii  {}", package)))
    }

    async fn update(&self) -> Result<()> {
        let output = tokio::process::Command::new("apt-get").args(["update"]).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("apt-get update failed: {}", stderr);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "apt"
    }
}

/// yum 包管理器（CentOS/RHEL/Fedora）
pub struct YumManager;

#[async_trait]
impl PackageManager for YumManager {
    async fn install(&self, package: &str) -> Result<()> {
        let output = tokio::process::Command::new("yum")
            .args(["install", "-y", package])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("yum install failed: {}", stderr);
        }

        Ok(())
    }

    async fn uninstall(&self, package: &str) -> Result<()> {
        let output = tokio::process::Command::new("yum")
            .args(["remove", "-y", package])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("yum remove failed: {}", stderr);
        }

        Ok(())
    }

    async fn is_installed(&self, package: &str) -> Result<bool> {
        let output = tokio::process::Command::new("rpm").args(["-q", package]).output().await?;

        Ok(output.status.success())
    }

    async fn update(&self) -> Result<()> {
        let output = tokio::process::Command::new("yum").args(["makecache"]).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("yum makecache failed: {}", stderr);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "yum"
    }
}

/// apk 包管理器（Alpine）
pub struct ApkManager;

#[async_trait]
impl PackageManager for ApkManager {
    async fn install(&self, package: &str) -> Result<()> {
        let output = tokio::process::Command::new("apk").args(["add", package]).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("apk add failed: {}", stderr);
        }

        Ok(())
    }

    async fn uninstall(&self, package: &str) -> Result<()> {
        let output = tokio::process::Command::new("apk").args(["del", package]).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("apk del failed: {}", stderr);
        }

        Ok(())
    }

    async fn is_installed(&self, package: &str) -> Result<bool> {
        let output = tokio::process::Command::new("apk")
            .args(["info", "-e", package])
            .output()
            .await?;

        Ok(output.status.success())
    }

    async fn update(&self) -> Result<()> {
        let output = tokio::process::Command::new("apk").args(["update"]).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("apk update failed: {}", stderr);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "apk"
    }
}

/// 检测当前系统的包管理器
pub fn detect_package_manager() -> Result<Box<dyn PackageManager>> {
    let os = std::env::consts::OS;

    match os {
        "linux" => {
            // 检测发行版
            if let Ok(distro) = detect_distro() {
                match distro.as_str() {
                    "ubuntu" | "debian" => Ok(Box::new(AptManager)),
                    "centos" | "rhel" | "fedora" => Ok(Box::new(YumManager)),
                    "alpine" => Ok(Box::new(ApkManager)),
                    _ => anyhow::bail!("Unsupported distribution: {}", distro),
                }
            } else {
                // 尝试检测包管理器
                if which("apt-get") {
                    Ok(Box::new(AptManager))
                } else if which("yum") {
                    Ok(Box::new(YumManager))
                } else if which("apk") {
                    Ok(Box::new(ApkManager))
                } else {
                    anyhow::bail!("Could not detect package manager")
                }
            }
        }
        "macos" => {
            if which("brew") {
                // TODO: 实现 HomebrewManager
                anyhow::bail!("Homebrew support not yet implemented")
            } else {
                anyhow::bail!("Homebrew not found")
            }
        }
        _ => anyhow::bail!("Unsupported OS: {}", os),
    }
}

/// 检测 Linux 发行版
fn detect_distro() -> Result<String> {
    let os_release = std::fs::read_to_string("/etc/os-release")?;
    for line in os_release.lines() {
        if line.starts_with("ID=") {
            let id = line[3..].trim_matches('"').to_string();
            return Ok(id);
        }
    }
    anyhow::bail!("Could not detect distribution")
}

/// 检测命令是否存在
fn which(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
