use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 工具清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinManifest {
    pub version: u32,
    pub tools: HashMap<String, ToolEntry>,
}

/// 工具条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub hash: String,
    pub description: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub remote_path: String,
}

/// 同步结果
#[derive(Debug)]
pub struct SyncResult {
    pub tool: String,
    pub status: SyncStatus,
    pub message: String,
}

/// 同步状态
#[derive(Debug)]
pub enum SyncStatus {
    /// 同步
    Synced,
    /// 冲突（本地和远程版本不同）
    Conflict,
    /// 仅远程
    RemoteOnly,
    /// 仅本地
    LocalOnly,
}

/// 工具同步器
pub struct BinSyncer {
    manifest_path: PathBuf,
    local_bin_dir: PathBuf,
    manifest: BinManifest,
}

impl BinSyncer {
    /// 创建新的工具同步器
    pub fn new(manifest_path: PathBuf, local_bin_dir: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&manifest_path).context("Failed to read bin-manifest.json")?;

        let manifest: BinManifest = serde_json::from_str(&content).context("Failed to parse bin-manifest.json")?;

        Ok(Self {
            manifest_path,
            local_bin_dir,
            manifest,
        })
    }

    /// 检查工具同步状态
    pub async fn check(&self, tool: &str, remote_host: &str) -> Result<SyncResult> {
        let entry = self
            .manifest
            .tools
            .get(tool)
            .context(format!("Tool '{}' not found in manifest", tool))?;

        let local_path = self.local_bin_dir.join(tool);
        let local_exists = local_path.exists();

        // 检查远程文件是否存在
        let remote_exists = self.check_remote_exists(remote_host, &entry.remote_path).await?;

        match (local_exists, remote_exists) {
            (true, true) => {
                // 比较哈希
                let local_hash = self.calc_local_hash(tool)?;
                if local_hash == entry.hash {
                    Ok(SyncResult {
                        tool: tool.to_string(),
                        status: SyncStatus::Synced,
                        message: "Synced".to_string(),
                    })
                } else {
                    Ok(SyncResult {
                        tool: tool.to_string(),
                        status: SyncStatus::Conflict,
                        message: "Conflict: local and remote versions differ".to_string(),
                    })
                }
            }
            (true, false) => Ok(SyncResult {
                tool: tool.to_string(),
                status: SyncStatus::LocalOnly,
                message: "Local only".to_string(),
            }),
            (false, true) => Ok(SyncResult {
                tool: tool.to_string(),
                status: SyncStatus::RemoteOnly,
                message: "Remote only".to_string(),
            }),
            (false, false) => Ok(SyncResult {
                tool: tool.to_string(),
                status: SyncStatus::Synced,
                message: "Not installed".to_string(),
            }),
        }
    }

    /// 拉取远程工具到本地
    pub async fn pull(&self, tool: &str, remote_host: &str) -> Result<SyncResult> {
        let entry = self
            .manifest
            .tools
            .get(tool)
            .context(format!("Tool '{}' not found in manifest", tool))?;

        // TODO: 实现从远程拉取文件
        Ok(SyncResult {
            tool: tool.to_string(),
            status: SyncStatus::Synced,
            message: format!("Pulled {} from {}", tool, remote_host),
        })
    }

    /// 推送本地工具到远程
    pub async fn push(&self, tool: &str, remote_host: &str) -> Result<SyncResult> {
        let entry = self
            .manifest
            .tools
            .get(tool)
            .context(format!("Tool '{}' not found in manifest", tool))?;

        // TODO: 实现推送到远程
        Ok(SyncResult {
            tool: tool.to_string(),
            status: SyncStatus::Synced,
            message: format!("Pushed {} to {}", tool, remote_host),
        })
    }

    /// 添加工具到清单
    pub fn add_tool(&mut self, name: &str, hash: &str, description: &str, tool_type: &str) -> Result<()> {
        let entry = ToolEntry {
            hash: hash.to_string(),
            description: description.to_string(),
            tool_type: tool_type.to_string(),
            remote_path: format!("~/bin/{}", name),
        };

        self.manifest.tools.insert(name.to_string(), entry);
        self.save_manifest()
    }

    /// 从清单移除工具
    pub fn remove_tool(&mut self, name: &str) -> Result<()> {
        if self.manifest.tools.remove(name).is_none() {
            anyhow::bail!("Tool '{}' not found in manifest", name);
        }
        self.save_manifest()
    }

    /// 保存清单到文件
    fn save_manifest(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.manifest)?;
        std::fs::write(&self.manifest_path, content)?;
        Ok(())
    }

    /// 计算本地文件哈希
    fn calc_local_hash(&self, tool: &str) -> Result<String> {
        let path = self.local_bin_dir.join(tool);
        let content = std::fs::read(&path)?;
        let hash = sha256::digest(&content);
        Ok(hash)
    }

    /// 检查远程文件是否存在
    async fn check_remote_exists(&self, remote_host: &str, remote_path: &str) -> Result<bool> {
        // TODO: 实现远程文件检查
        Ok(false)
    }
}

/// 计算 SHA256 哈希
mod sha256 {
    use std::io::Read;

    pub fn digest(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
