use crate::api_types::ApiKeyData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

// ========== 定数 ==========
const ADMIN_API_KEY: &str = "changeme";
const API_KEYS_FILE: &str = "api_keys.json";

// ========== エラー型 ==========
#[derive(Debug)]
pub enum ApiKeyError {
    AdminRequired,
    KeyNotFound,
    KeyAlreadyExists,
    FileError(String),
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiKeyError::AdminRequired => write!(f, "管理者権限が必要です"),
            ApiKeyError::KeyNotFound => write!(f, "APIキーが見つかりません"),
            ApiKeyError::KeyAlreadyExists => write!(f, "APIキーが既に存在します"),
            ApiKeyError::FileError(msg) => write!(f, "ファイルエラー: {}", msg),
        }
    }
}

impl std::error::Error for ApiKeyError {}

// ========== APIキーストア ==========
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
        if Path::new(API_KEYS_FILE).exists() {
            match fs::read_to_string(API_KEYS_FILE) {
                Ok(content) => {
                    Self::parse_from_string(&content).unwrap_or_else(Self::new)
                }
                Err(_) => Self::new(),
            }
        } else {
            Self::new()
        }
    }

    fn parse_from_string(content: &str) -> Option<Self> {
        // 新しい形式で解析を試行
        if let Ok(store) = serde_json::from_str::<Self>(content) {
            return Some(store);
        }

        // 古い形式からの移行を試行
        Self::migrate_from_legacy_format(content)
    }

    fn migrate_from_legacy_format(content: &str) -> Option<Self> {
        if let Ok(legacy_data) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(keys_array) = legacy_data.get("keys").and_then(|v| v.as_array()) {
                let mut keys = HashMap::new();
                let now = chrono::Utc::now().to_rfc3339();
                
                for key_value in keys_array {
                    if let Some(key_str) = key_value.as_str() {
                        let api_key_data = ApiKeyData::new(key_str.to_string(), now.clone());
                        keys.insert(key_str.to_string(), api_key_data);
                    }
                }
                return Some(Self { keys });
            }
        }
        None
    }

    pub fn save_to_file(&self) -> Result<(), ApiKeyError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ApiKeyError::FileError(format!("JSON serialization failed: {}", e)))?;
        
        fs::write(API_KEYS_FILE, content)
            .map_err(|e| ApiKeyError::FileError(format!("File write failed: {}", e)))?;
        
        Ok(())
    }

    // ========== 基本操作 ==========

    pub fn add_key(&mut self, key: String) -> Result<(), ApiKeyError> {
        if self.keys.contains_key(&key) {
            return Err(ApiKeyError::KeyAlreadyExists);
        }

        let api_key_data = ApiKeyData::new(key.clone(), chrono::Utc::now().to_rfc3339());
        self.keys.insert(key, api_key_data);
        self.save_to_file()?;
        Ok(())
    }

    pub fn validate_key(&self, key: &str) -> bool {
        self.keys.contains_key(key)
    }

    pub fn validate_admin_key(&self, admin_key: &str) -> bool {
        admin_key == ADMIN_API_KEY
    }

    pub fn remove_key(&mut self, admin_key: &str, key: &str) -> Result<(), ApiKeyError> {
        if !self.validate_admin_key(admin_key) {
            return Err(ApiKeyError::AdminRequired);
        }

        if self.keys.remove(key).is_some() {
            self.save_to_file()?;
            Ok(())
        } else {
            Err(ApiKeyError::KeyNotFound)
        }
    }

    // ========== 使用量管理 ==========

    pub fn add_usage(&mut self, key: &str, original_bytes: u64, processed_bytes: u64) -> Result<(), ApiKeyError> {
        if let Some(api_key_data) = self.keys.get_mut(key) {
            api_key_data.add_usage(original_bytes, processed_bytes);
            self.save_to_file()?;
            Ok(())
        } else {
            Err(ApiKeyError::KeyNotFound)
        }
    }

    pub fn get_usage(&self, key: &str) -> Option<u64> {
        self.keys.get(key).map(|data| data.total_bytes_processed)
    }

    // ========== データ取得 ==========

    pub fn list_keys(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    pub fn list_keys_with_data(&self, admin_key: &str) -> Result<Vec<ApiKeyData>, ApiKeyError> {
        if !self.validate_admin_key(admin_key) {
            return Err(ApiKeyError::AdminRequired);
        }
        Ok(self.keys.values().cloned().collect())
    }

    pub fn get_statistics(&self, admin_key: &str) -> Result<StatisticsData, ApiKeyError> {
        if !self.validate_admin_key(admin_key) {
            return Err(ApiKeyError::AdminRequired);
        }

        let total_original: u64 = self.keys.values().map(|k| k.total_original_bytes).sum();
        let total_processed: u64 = self.keys.values().map(|k| k.total_processed_bytes).sum();
        let total_compressions: u64 = self.keys.values().map(|k| k.compression_count).sum();
        let total_keys = self.keys.len();
        let compression_ratio = if total_original > 0 {
            ((total_original - total_processed) * 100) / total_original
        } else {
            0
        };
        
        Ok(StatisticsData {
            total_original_bytes: total_original,
            total_processed_bytes: total_processed,
            total_compressions,
            compression_ratio,
            total_keys,
        })
    }

    // ========== 設定取得 ==========

    pub fn get_admin_key() -> &'static str {
        ADMIN_API_KEY
    }
}

// ========== 統計データ構造 ==========
#[derive(Debug, Clone)]
pub struct StatisticsData {
    pub total_original_bytes: u64,
    pub total_processed_bytes: u64,
    pub total_compressions: u64,
    pub compression_ratio: u64,
    pub total_keys: usize,
}

impl StatisticsData {
    pub fn as_tuple(&self) -> (u64, u64, u64, u64, usize) {
        (
            self.total_original_bytes,
            self.total_processed_bytes,
            self.total_compressions,
            self.compression_ratio,
            self.total_keys,
        )
    }
}

// ========== 型エイリアス ==========
pub type SharedApiKeyStore = Arc<RwLock<ApiKeyStore>>;

// ========== APIキーデータの拡張 ==========
impl ApiKeyData {
    pub fn new(key: String, created_at: String) -> Self {
        Self {
            key,
            total_bytes_processed: 0,
            total_original_bytes: 0,
            total_processed_bytes: 0,
            compression_count: 0,
            created_at,
            last_used: None,
        }
    }

    pub fn add_usage(&mut self, original_bytes: u64, processed_bytes: u64) {
        self.total_bytes_processed += original_bytes;
        self.total_original_bytes += original_bytes;
        self.total_processed_bytes += processed_bytes;
        self.compression_count += 1;
        self.last_used = Some(chrono::Utc::now().to_rfc3339());
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.total_original_bytes > 0 {
            ((self.total_original_bytes - self.total_processed_bytes) as f64 / self.total_original_bytes as f64) * 100.0
        } else {
            0.0
        }
    }
}

