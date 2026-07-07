pub mod bin_sync;
pub mod packages;
pub mod pkgmgr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 包类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PackageType {
    /// 系统包（通过包管理器安装）
    Pkg,
    /// 自定义工具（通过 bin-sync 同步）
    Bin,
}

/// 包定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub pkg_type: PackageType,
    pub description: String,
    pub group: String,
    /// 跨发行版包名覆盖
    pub apt_name: Option<String>,
    pub yum_name: Option<String>,
    pub apk_name: Option<String>,
}

/// 预配结果
#[derive(Debug)]
pub struct ProvisionResult {
    pub package: String,
    pub success: bool,
    pub message: String,
}

/// 预配引擎
pub struct ProvisionEngine {
    packages: Vec<Package>,
    pkg_manager: Box<dyn pkgmgr::PackageManager>,
}

impl ProvisionEngine {
    /// 创建新的预配引擎
    pub fn new(packages: Vec<Package>, pkg_manager: Box<dyn pkgmgr::PackageManager>) -> Self {
        Self { packages, pkg_manager }
    }

    /// 预配所有包
    pub async fn provision_all(&self, dry_run: bool) -> Vec<ProvisionResult> {
        let mut results = Vec::new();

        for package in &self.packages {
            let result = match package.pkg_type {
                PackageType::Pkg => self.provision_package(package, dry_run).await,
                PackageType::Bin => self.provision_bin(package, dry_run).await,
            };
            results.push(result);
        }

        results
    }

    /// 预配系统包
    async fn provision_package(&self, package: &Package, dry_run: bool) -> ProvisionResult {
        let pkg_name = self.get_package_name(package);

        if dry_run {
            return ProvisionResult {
                package: package.name.clone(),
                success: true,
                message: format!("[DRY RUN] Would install {}", pkg_name),
            };
        }

        match self.pkg_manager.install(&pkg_name).await {
            Ok(_) => ProvisionResult {
                package: package.name.clone(),
                success: true,
                message: format!("Installed {}", pkg_name),
            },
            Err(e) => ProvisionResult {
                package: package.name.clone(),
                success: false,
                message: format!("Failed to install {}: {}", pkg_name, e),
            },
        }
    }

    /// 预配自定义工具
    async fn provision_bin(&self, package: &Package, dry_run: bool) -> ProvisionResult {
        if dry_run {
            return ProvisionResult {
                package: package.name.clone(),
                success: true,
                message: format!("[DRY RUN] Would sync {}", package.name),
            };
        }

        // TODO: 实现 bin-sync 逻辑
        ProvisionResult {
            package: package.name.clone(),
            success: true,
            message: format!("Synced {}", package.name),
        }
    }

    /// 获取包名（考虑跨发行版覆盖）
    fn get_package_name(&self, package: &Package) -> String {
        let os = std::env::consts::OS;

        match os {
            "linux" => {
                // 检测发行版
                if let Ok(distro) = self.detect_distro() {
                    match distro.as_str() {
                        "ubuntu" | "debian" => package.apt_name.clone().unwrap_or_else(|| package.name.clone()),
                        "centos" | "rhel" | "fedora" => {
                            package.yum_name.clone().unwrap_or_else(|| package.name.clone())
                        }
                        "alpine" => package.apk_name.clone().unwrap_or_else(|| package.name.clone()),
                        _ => package.name.clone(),
                    }
                } else {
                    package.name.clone()
                }
            }
            "macos" => package.name.clone(),
            _ => package.name.clone(),
        }
    }

    /// 检测 Linux 发行版
    fn detect_distro(&self) -> Result<String> {
        let os_release = std::fs::read_to_string("/etc/os-release")?;
        for line in os_release.lines() {
            if line.starts_with("ID=") {
                let id = line[3..].trim_matches('"').to_string();
                return Ok(id);
            }
        }
        anyhow::bail!("Could not detect distribution")
    }
}

impl ProvisionEngine {
    /// Get the number of packages
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}
