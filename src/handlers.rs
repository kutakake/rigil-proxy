use crate::api_key::SharedApiKeyStore;
use crate::api_types::{ApiResponse, UsageResponse};
use crate::html_parser::{get_base_url, get_html, normalize_url, parse_html_to_text};

use hyper::{Body, Request, Response, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use serde_json;

// 共通のレスポンス生成
pub fn create_html_response(body: String) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
    response
}

pub fn create_json_response(body: String, status: StatusCode) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
    response
}

// クエリパラメータ解析
pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

// 簡単なエラーページ生成
pub fn create_error_page(message: &str) -> String {
    format!(
        r#"<html><head><meta charset="UTF-8"></head><body>
            <h1>エラー</h1>
            <p>{}</p>
            <p><a href="/">ホーム画面に戻る</a></p>
        </body></html>"#,
        message
    )
}

// APIキー検証（簡素化版）
async fn validate_api_key(params: &HashMap<String, String>, api_key_store: &SharedApiKeyStore) -> Option<String> {
    if let Some(api_key) = params.get("api_key") {
        let store = api_key_store.read().await;
        if store.validate_key(api_key) {
            return Some(api_key.clone());
        }
    }
    None
}

// 管理者キー検証
async fn validate_admin_key(params: &HashMap<String, String>, api_key_store: &SharedApiKeyStore) -> bool {
    if let Some(admin_key) = params.get("admin_key") {
        let store = api_key_store.read().await;
        return store.validate_admin_key(admin_key);
    }
    false
}

// プロキシリクエストハンドラー（統計機能付き）
pub async fn handle_proxy_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    let target_url = match params.get("url") {
        Some(url) => url,
        None => {
            let error_html = create_error_page("URLパラメータが必要です");
            return Ok(create_html_response(error_html));
        }
    };

    let api_key = match validate_api_key(&params, &api_key_store).await {
        Some(key) => key,
        None => {
            let error_html = create_error_page("有効なAPIキーが必要です");
            return Ok(create_html_response(error_html));
        }
    };

    let normalized_url = normalize_url(target_url);

    match get_html(&normalized_url).await {
        Ok((html_body, final_url)) => {
            let base_url = get_base_url(&final_url);
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);
            let processed_size = processed_html.len() as u64;

            // 使用量を記録
            let mut store = api_key_store.write().await;
            store.add_usage(&api_key, original_size, processed_size);
            drop(store);

            Ok(create_html_response(processed_html))
        }
        Err(e) => {
            let error_html = create_error_page(&format!("URL取得エラー: {}", e));
            Ok(create_html_response(error_html))
        }
    }
}

// JSON API（統計機能付き）
pub async fn handle_api_get_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    let target_url = match params.get("url") {
        Some(url) => url,
        None => {
            let error_response = ApiResponse {
                success: false,
                data: None,
                error: Some("URLパラメータが必要です".to_string()),
                original_url: None,
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            return Ok(create_json_response(json_response, StatusCode::BAD_REQUEST));
        }
    };

    let api_key = match validate_api_key(&params, &api_key_store).await {
        Some(key) => key,
        None => {
            let error_response = ApiResponse {
                success: false,
                data: None,
                error: Some("有効なAPIキーが必要です".to_string()),
                original_url: Some(target_url.to_string()),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            return Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED));
        }
    };

    let normalized_url = normalize_url(target_url);

    let response = match get_html(&normalized_url).await {
        Ok((html_body, final_url)) => {
            let base_url = get_base_url(&final_url);
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);
            let processed_size = processed_html.len() as u64;

            // 使用量を記録
            let mut store = api_key_store.write().await;
            store.add_usage(&api_key, original_size, processed_size);
            drop(store);

            ApiResponse {
                success: true,
                data: Some(processed_html),
                error: None,
                original_url: Some(final_url),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: Some(original_size),
                processed_size_bytes: Some(processed_size),
            }
        }
        Err(e) => {
            ApiResponse {
                success: false,
                data: None,
                error: Some(e),
                original_url: Some(normalized_url),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            }
        }
    };

    let json_response = serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"success":false,"data":null,"error":"JSON serialization error"}"#.to_string()
    });

    Ok(create_json_response(json_response, StatusCode::OK))
}

// 簡単なレスポンス型（管理機能用）
#[derive(serde::Serialize)]
struct SimpleResponse {
    success: bool,
    message: Option<String>,
    keys: Option<Vec<String>>,
    error: Option<String>,
}

// APIキー作成ハンドラー（管理者権限不要）
pub async fn handle_create_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes);

    // リクエストボディをパース
    let request_data: Result<serde_json::Value, _> = serde_json::from_str(&body_str);
    
    match request_data {
        Ok(data) => {
            if let Some(key) = data.get("key").and_then(|k| k.as_str()) {
                let mut store = api_key_store.write().await;
                
                // キーが既に存在するかチェック
                if store.validate_key(key) {
                    let error_response = SimpleResponse {
                        success: false,
                        message: None,
                        keys: None,
                        error: Some("APIキーが既に存在します".to_string()),
                    };
                    let json_response = serde_json::to_string(&error_response).unwrap();
                    return Ok(create_json_response(json_response, StatusCode::CONFLICT));
                }
                
                store.add_key(key.to_string());
                drop(store);
                
                let success_response = SimpleResponse {
                    success: true,
                    message: Some(format!("APIキー '{}' を作成しました", key)),
                    keys: None,
                    error: None,
                };
                let json_response = serde_json::to_string(&success_response).unwrap();
                Ok(create_json_response(json_response, StatusCode::OK))
            } else {
                let error_response = SimpleResponse {
                    success: false,
                    message: None,
                    keys: None,
                    error: Some("keyフィールドが必要です".to_string()),
                };
                let json_response = serde_json::to_string(&error_response).unwrap();
                Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
            }
        }
        Err(_) => {
            let error_response = SimpleResponse {
                success: false,
                message: None,
                keys: None,
                error: Some("無効なJSONです".to_string()),
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
        }
    }
}

// APIキー一覧取得ハンドラー（管理者認証必要）
pub async fn handle_list_keys_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if !validate_admin_key(&params, &api_key_store).await {
        let error_response = UsageResponse {
            success: false,
            key: None,
            total_bytes_processed: None,
            keys: None,
            error: Some("管理者権限が必要です".to_string()),
        };
        let json_response = serde_json::to_string(&error_response).unwrap();
        return Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED));
    }

    let admin_key = params.get("admin_key").unwrap();
    let store = api_key_store.read().await;
    match store.list_keys_with_data(admin_key) {
        Ok(keys_data) => {
            let response = UsageResponse {
                success: true,
                key: None,
                total_bytes_processed: None,
                keys: Some(keys_data),
                error: None,
            };
            let json_response = serde_json::to_string(&response).unwrap();
            Ok(create_json_response(json_response, StatusCode::OK))
        }
        Err(err_msg) => {
            let error_response = UsageResponse {
                success: false,
                key: None,
                total_bytes_processed: None,
                keys: None,
                error: Some(err_msg),
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED))
        }
    }
}

// APIキー削除ハンドラー（管理者認証必要）
pub async fn handle_delete_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if !validate_admin_key(&params, &api_key_store).await {
        let error_response = SimpleResponse {
            success: false,
            message: None,
            keys: None,
            error: Some("管理者権限が必要です".to_string()),
        };
        let json_response = serde_json::to_string(&error_response).unwrap();
        return Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED));
    }

    if let Some(key_to_delete) = params.get("key") {
        let admin_key = params.get("admin_key").unwrap();
        let mut store = api_key_store.write().await;
        
        match store.remove_key(admin_key, key_to_delete) {
            Ok(()) => {
                let success_response = SimpleResponse {
                    success: true,
                    message: Some(format!("APIキー '{}' を削除しました", key_to_delete)),
                    keys: None,
                    error: None,
                };
                let json_response = serde_json::to_string(&success_response).unwrap();
                Ok(create_json_response(json_response, StatusCode::OK))
            }
            Err(err_msg) => {
                let error_response = SimpleResponse {
                    success: false,
                    message: None,
                    keys: None,
                    error: Some(err_msg),
                };
                let json_response = serde_json::to_string(&error_response).unwrap();
                Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED))
            }
        }
    } else {
        let error_response = SimpleResponse {
            success: false,
            message: None,
            keys: None,
            error: Some("keyパラメータが必要です".to_string()),
        };
        let json_response = serde_json::to_string(&error_response).unwrap();
        Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
    }
}

// 統計取得ハンドラー（管理者認証必要）
pub async fn handle_statistics_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if !validate_admin_key(&params, &api_key_store).await {
        let response = serde_json::json!({
            "success": false,
            "error": "管理者権限が必要です"
        });
        let json_response = serde_json::to_string(&response).unwrap();
        return Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED));
    }

    let admin_key = params.get("admin_key").unwrap();
    let store = api_key_store.read().await;
    match store.get_statistics(admin_key) {
        Ok((total_original, total_processed, total_compressions, compression_ratio, total_keys)) => {
            let response = serde_json::json!({
                "success": true,
                "statistics": {
                    "total_keys": total_keys,
                    "total_original_bytes": total_original,
                    "total_processed_bytes": total_processed,
                    "total_compressions": total_compressions,
                    "compression_ratio": compression_ratio
                }
            });
            let json_response = serde_json::to_string(&response).unwrap();
            Ok(create_json_response(json_response, StatusCode::OK))
        }
        Err(err_msg) => {
            let response = serde_json::json!({
                "success": false,
                "error": err_msg
            });
            let json_response = serde_json::to_string(&response).unwrap();
            Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED))
        }
    }
}

// 管理者ログイン検証ハンドラー
pub async fn handle_admin_login_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes);

    let request_data: Result<serde_json::Value, _> = serde_json::from_str(&body_str);
    
    match request_data {
        Ok(data) => {
            if let Some(admin_key) = data.get("admin_key").and_then(|k| k.as_str()) {
                let store = api_key_store.read().await;
                
                if store.validate_admin_key(admin_key) {
                    let success_response = serde_json::json!({
                        "success": true,
                        "message": "ログイン成功"
                    });
                    let json_response = serde_json::to_string(&success_response).unwrap();
                    Ok(create_json_response(json_response, StatusCode::OK))
                } else {
                    let error_response = serde_json::json!({
                        "success": false,
                        "error": "無効な管理者キーです"
                    });
                    let json_response = serde_json::to_string(&error_response).unwrap();
                    Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED))
                }
            } else {
                let error_response = serde_json::json!({
                    "success": false,
                    "error": "admin_keyフィールドが必要です"
                });
                let json_response = serde_json::to_string(&error_response).unwrap();
                Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
            }
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "success": false,
                "error": "無効なJSONです"
            });
            let json_response = serde_json::to_string(&error_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
        }
    }
} 