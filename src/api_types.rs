use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKeyData {
    pub key: String,
    pub total_bytes_processed: u64,
    pub total_original_bytes: u64,
    pub total_processed_bytes: u64,
    pub compression_count: u64,
    pub created_at: String,
    pub last_used: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
    pub original_url: Option<String>,
    pub processed_at: String,
    pub original_size_bytes: Option<u64>,
    pub processed_size_bytes: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiRequest {
    pub url: String,
    pub format: Option<String>, // "html" or "json"
}

#[derive(Serialize, Deserialize)]
pub struct CreateKeyRequest {
    pub admin_key: String,
    pub key: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsageResponse {
    pub success: bool,
    pub key: Option<String>,
    pub total_bytes_processed: Option<u64>,
    pub keys: Option<Vec<ApiKeyData>>,
    pub error: Option<String>,
} 