use crate::api_types::ApiKeyData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

const ADMIN_API_KEY: &str = "changeme";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKeyStore {
    keys: HashMap<String, ApiKeyData>,
}

impl ApiKeyStore {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn load_from_file() -> Self {
        if Path::new("api_keys.json").exists() {
            match fs::read_to_string("api_keys.json") {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(store) => store,
                        Err(_) => {
                            // 古い形式から新しい形式に変換を試行
                            Self::migrate_from_old_format(&content).unwrap_or_else(|| Self::new())
                        }
                    }
                }
                Err(_) => Self::new(),
            }
        } else {
            Self::new()
        }
    }

    // 古い形式のデータから移行（統計機能付き）
    fn migrate_from_old_format(content: &str) -> Option<Self> {
        // シンプルな形式からの移行
        if let Ok(simple_data) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(keys_array) = simple_data.get("keys").and_then(|v| v.as_array()) {
                let mut keys = HashMap::new();
                for key in keys_array {
                    if let Some(key_str) = key.as_str() {
                        let api_key_data = ApiKeyData {
                            key: key_str.to_string(),
                            total_bytes_processed: 0,
                            total_original_bytes: 0,
                            total_processed_bytes: 0,
                            compression_count: 0,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            last_used: None,
                        };
                        keys.insert(key_str.to_string(), api_key_data);
                    }
                }
                return Some(Self { keys });
            }
        }
        None
    }

    pub fn save_to_file(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write("api_keys.json", content);
        }
    }

    pub fn add_key(&mut self, key: String) {
        let api_key_data = ApiKeyData {
            key: key.clone(),
            total_bytes_processed: 0,
            total_original_bytes: 0,
            total_processed_bytes: 0,
            compression_count: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };
        self.keys.insert(key, api_key_data);
        self.save_to_file();
    }

    pub fn validate_key(&self, key: &str) -> bool {
        self.keys.contains_key(key)
    }

    pub fn validate_admin_key(&self, admin_key: &str) -> bool {
        admin_key == ADMIN_API_KEY
    }

    pub fn add_usage(&mut self, key: &str, original_bytes: u64, processed_bytes: u64) {
        if let Some(api_key_data) = self.keys.get_mut(key) {
            api_key_data.total_bytes_processed += original_bytes;
            api_key_data.total_original_bytes += original_bytes;
            api_key_data.total_processed_bytes += processed_bytes;
            api_key_data.compression_count += 1;
            api_key_data.last_used = Some(chrono::Utc::now().to_rfc3339());
            self.save_to_file();
        }
    }

    pub fn get_usage(&self, key: &str) -> Option<u64> {
        self.keys.get(key).map(|data| data.total_bytes_processed)
    }

    pub fn remove_key(&mut self, admin_key: &str, key: &str) -> Result<(), String> {
        if !self.validate_admin_key(admin_key) {
            return Err("管理者権限が必要です".to_string());
        }

        if self.keys.remove(key).is_some() {
            self.save_to_file();
            Ok(())
        } else {
            Err("APIキーが見つかりません".to_string())
        }
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    pub fn list_keys_with_data(&self, admin_key: &str) -> Result<Vec<ApiKeyData>, String> {
        if !self.validate_admin_key(admin_key) {
            return Err("管理者権限が必要です".to_string());
        }
        Ok(self.keys.values().cloned().collect())
    }

    pub fn get_statistics(&self, admin_key: &str) -> Result<(u64, u64, u64, u64, usize), String> {
        if !self.validate_admin_key(admin_key) {
            return Err("管理者権限が必要です".to_string());
        }

        let total_original = self.keys.values().map(|k| k.total_original_bytes).sum();
        let total_processed = self.keys.values().map(|k| k.total_processed_bytes).sum();
        let total_compressions = self.keys.values().map(|k| k.compression_count).sum();
        let total_keys = self.keys.len();
        let compression_ratio = if total_original > 0 {
            ((total_original - total_processed) * 100) / total_original
        } else {
            0
        };
        
        Ok((total_original, total_processed, total_compressions, compression_ratio, total_keys))
    }

    pub fn get_admin_key() -> &'static str {
        ADMIN_API_KEY
    }
}

pub type SharedApiKeyStore = Arc<RwLock<ApiKeyStore>>;
