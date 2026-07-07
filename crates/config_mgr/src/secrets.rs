use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use anyhow::{Result, bail};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 加密的秘密存储
#[derive(Debug, Serialize, Deserialize)]
pub struct SecretsStore {
    /// 加密的秘密数据
    secrets: HashMap<String, Vec<u8>>,
    /// 随机数（nonce）
    nonce: Vec<u8>,
}

/// 秘密管理器
pub struct SecretsManager {
    store_path: PathBuf,
    cipher: Aes256Gcm,
}

impl SecretsManager {
    /// 创建新的秘密管理器
    pub fn new(store_path: PathBuf, master_password: &str) -> Result<Self> {
        // 从主密码派生密钥
        let key = derive_key(master_password);
        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!("Failed to create cipher: {:?}", e))?;

        Ok(Self { store_path, cipher })
    }

    /// 加载秘密存储
    pub fn load(&self) -> Result<HashMap<String, String>> {
        if !self.store_path.exists() {
            return Ok(HashMap::new());
        }

        let content = std::fs::read_to_string(&self.store_path)?;
        let store: SecretsStore = toml::from_str(&content)?;

        let mut secrets = HashMap::new();
        let nonce = Nonce::from_slice(&store.nonce);

        for (key, encrypted_value) in &store.secrets {
            let decrypted = self
                .cipher
                .decrypt(nonce, encrypted_value.as_ref())
                .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

            let value = String::from_utf8(decrypted)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in decrypted secret: {}", e))?;

            secrets.insert(key.clone(), value);
        }

        Ok(secrets)
    }

    /// 保存秘密到存储
    pub fn save(&self, secrets: &HashMap<String, String>) -> Result<()> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut encrypted_secrets = HashMap::new();
        for (key, value) in secrets {
            let encrypted = self
                .cipher
                .encrypt(nonce, value.as_bytes())
                .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

            encrypted_secrets.insert(key.clone(), encrypted);
        }

        let store = SecretsStore {
            secrets: encrypted_secrets,
            nonce: nonce_bytes.to_vec(),
        };

        let content = toml::to_string_pretty(&store)?;
        std::fs::write(&self.store_path, content)?;

        Ok(())
    }

    /// 设置单个秘密
    pub fn set_secret(&self, key: &str, value: &str) -> Result<()> {
        let mut secrets = self.load()?;
        secrets.insert(key.to_string(), value.to_string());
        self.save(&secrets)
    }

    /// 获取单个秘密
    pub fn get_secret(&self, key: &str) -> Result<Option<String>> {
        let secrets = self.load()?;
        Ok(secrets.get(key).cloned())
    }

    /// 删除单个秘密
    pub fn delete_secret(&self, key: &str) -> Result<()> {
        let mut secrets = self.load()?;
        if secrets.remove(key).is_none() {
            bail!("Secret '{}' not found", key);
        }
        self.save(&secrets)
    }
}

/// 从主密码派生 256 位密钥
fn derive_key(password: &str) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    let hash = hasher.finish();

    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&hash.to_be_bytes());
    key[8..16].copy_from_slice(&hash.to_le_bytes());
    key[16..24].copy_from_slice(&hash.to_be_bytes());
    key[24..32].copy_from_slice(&hash.to_le_bytes());

    key
}
