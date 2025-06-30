use crate::api_types::ApiKeyData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKeyStore {
    keys: HashMap<String, ApiKeyData>,
}

// … なんとなく、この鍵は特別
const ADMIN_API_KEY: &str = "admin_rigil_proxy_master_key_2024";

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
                        Err(_) => Self::new(),
                    }
                }
                Err(_) => Self::new(),
            }
        } else {
            Self::new()
        }
    }

    pub fn save_to_file(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write("api_keys.json", content);
        }
    }

    pub fn add_key(&mut self, admin_key: &str, key: String) -> Result<(), String> {
        if admin_key != ADMIN_API_KEY {
            return Err("管理者キーが必要だよ".to_string());
        }

        let api_key_data = ApiKeyData {
            key: key.clone(),
            total_bytes_processed: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };
        self.keys.insert(key, api_key_data);
        self.save_to_file();
        Ok(())
    }

    pub fn validate_key(&self, key: &str) -> bool {
        self.keys.contains_key(key)
    }

    pub fn add_usage(&mut self, key: &str, bytes: u64) {
        if let Some(api_key_data) = self.keys.get_mut(key) {
            println!("{}bytes", bytes);
            api_key_data.total_bytes_processed += bytes;
            api_key_data.last_used = Some(chrono::Utc::now().to_rfc3339());
            self.save_to_file();
        }
    }

    pub fn get_usage(&self, key: &str) -> Option<u64> {
        self.keys.get(key).map(|data| data.total_bytes_processed)
    }

    pub fn remove_key(&mut self, admin_key: &str, key: &str) -> Result<(), String> {
        if admin_key != ADMIN_API_KEY {
            return Err("管理者キーが必要だよ".to_string());
        }

        if self.keys.remove(key).is_some() {
            self.save_to_file();
            Ok(())
        } else {
            Err("そのキー、見つからないみたい".to_string())
        }
    }

    pub fn list_keys(&self, admin_key: &str) -> Result<Vec<ApiKeyData>, String> {
        if admin_key != ADMIN_API_KEY {
            return Err("管理者キーが必要だよ".to_string());
        }
        Ok(self.keys.values().cloned().collect())
    }
}

pub type SharedApiKeyStore = Arc<RwLock<ApiKeyStore>>;
