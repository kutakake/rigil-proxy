use crate::api_key::{SharedApiKeyStore, ApiKeyError};
use crate::api_types::{ApiResponse, UsageResponse};
use crate::html_parser::{get_base_url, get_html, normalize_url, parse_html_to_text};

use hyper::{Body, Request, Response, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use serde_json;

// ========== 共通ユーティリティ ==========

pub fn create_html_response(body: String) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
    response
}

pub fn create_json_response(body: String, status: StatusCode) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
    response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    response
}

pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

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

// ========== 認証ヘルパー ==========

async fn validate_api_key(params: &HashMap<String, String>, api_key_store: &SharedApiKeyStore) -> Option<String> {
    if let Some(api_key) = params.get("api_key") {
        let store = api_key_store.read().await;
        if store.validate_key(api_key) {
            return Some(api_key.clone());
        }
    }
    None
}

async fn validate_admin_key(params: &HashMap<String, String>, api_key_store: &SharedApiKeyStore) -> bool {
    if let Some(admin_key) = params.get("admin_key") {
        let store = api_key_store.read().await;
        return store.validate_admin_key(admin_key);
    }
    false
}

fn create_unauthorized_json_response(message: &str) -> Response<Body> {
    let error_response = serde_json::json!({
        "success": false,
        "error": message
    });
    let json_response = serde_json::to_string(&error_response).unwrap();
    create_json_response(json_response, StatusCode::UNAUTHORIZED)
}

fn create_bad_request_json_response(message: &str) -> Response<Body> {
    let error_response = serde_json::json!({
        "success": false,
        "error": message
    });
    let json_response = serde_json::to_string(&error_response).unwrap();
    create_json_response(json_response, StatusCode::BAD_REQUEST)
}

// ========== プロキシ機能 ==========

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

    match process_url_and_record_usage(target_url, &api_key, &api_key_store).await {
        Ok(processed_html) => Ok(create_html_response(processed_html)),
        Err(error_msg) => {
            let error_html = create_error_page(&error_msg);
            Ok(create_html_response(error_html))
        }
    }
}

pub async fn handle_api_get_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    let target_url = match params.get("url") {
        Some(url) => url,
        None => {
            let error_response = create_api_error_response("URLパラメータが必要です", None);
            return Ok(create_json_response(serde_json::to_string(&error_response).unwrap(), StatusCode::BAD_REQUEST));
        }
    };

    let api_key = match validate_api_key(&params, &api_key_store).await {
        Some(key) => key,
        None => {
            let error_response = create_api_error_response("有効なAPIキーが必要です", Some(target_url));
            return Ok(create_json_response(serde_json::to_string(&error_response).unwrap(), StatusCode::UNAUTHORIZED));
        }
    };

    let response = process_url_for_api(target_url, &api_key, &api_key_store).await;
    let json_response = serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"success":false,"data":null,"error":"JSON serialization error"}"#.to_string()
    });

    Ok(create_json_response(json_response, StatusCode::OK))
}

// ========== APIキー管理 ==========

#[derive(serde::Serialize)]
struct SimpleResponse {
    success: bool,
    message: Option<String>,
    error: Option<String>,
}

pub async fn handle_create_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let body_str = match get_request_body(req).await {
        Ok(body) => body,
        Err(response) => return Ok(response),
    };

    let request_data: Result<serde_json::Value, _> = serde_json::from_str(&body_str);

    match request_data {
        Ok(data) => {
            if let Some(key) = data.get("key").and_then(|k| k.as_str()) {
                match create_new_api_key(key, &api_key_store).await {
                    Ok(message) => {
                        let response = SimpleResponse {
                            success: true,
                            message: Some(message),
                            error: None,
                        };
                        Ok(create_json_response(serde_json::to_string(&response).unwrap(), StatusCode::OK))
                    }
                    Err(error) => {
                        let response = SimpleResponse {
                            success: false,
                            message: None,
                            error: Some(error.to_string()),
                        };
                        let status = match error {
                            ApiKeyError::KeyAlreadyExists => StatusCode::CONFLICT,
                            _ => StatusCode::INTERNAL_SERVER_ERROR,
                        };
                        Ok(create_json_response(serde_json::to_string(&response).unwrap(), status))
                    }
                }
            } else {
                Ok(create_bad_request_json_response("keyフィールドが必要です"))
            }
        }
        Err(_) => Ok(create_bad_request_json_response("無効なJSONです"))
    }
}

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
        return Ok(create_json_response(serde_json::to_string(&error_response).unwrap(), StatusCode::UNAUTHORIZED));
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
            Ok(create_json_response(serde_json::to_string(&response).unwrap(), StatusCode::OK))
        }
        Err(error) => Ok(create_unauthorized_json_response(&error.to_string()))
    }
}

pub async fn handle_delete_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if !validate_admin_key(&params, &api_key_store).await {
        return Ok(create_unauthorized_json_response("管理者権限が必要です"));
    }

    if let Some(key_to_delete) = params.get("key") {
        let admin_key = params.get("admin_key").unwrap();
        let mut store = api_key_store.write().await;

        match store.remove_key(admin_key, key_to_delete) {
            Ok(()) => {
                let response = SimpleResponse {
                    success: true,
                    message: Some(format!("APIキー '{}' を削除しました", key_to_delete)),
                    error: None,
                };
                Ok(create_json_response(serde_json::to_string(&response).unwrap(), StatusCode::OK))
            }
            Err(error) => Ok(create_unauthorized_json_response(&error.to_string()))
        }
    } else {
        Ok(create_bad_request_json_response("keyパラメータが必要です"))
    }
}

// ========== 統計機能 ==========

pub async fn handle_statistics_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if !validate_admin_key(&params, &api_key_store).await {
        return Ok(create_unauthorized_json_response("管理者権限が必要です"));
    }

    let admin_key = params.get("admin_key").unwrap();
    let store = api_key_store.read().await;
    match store.get_statistics(admin_key) {
        Ok(stats) => {
            let (total_original, total_processed, total_compressions, compression_ratio, total_keys) = stats.as_tuple();
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
            Ok(create_json_response(serde_json::to_string(&response).unwrap(), StatusCode::OK))
        }
        Err(error) => Ok(create_unauthorized_json_response(&error.to_string()))
    }
}

// ========== 認証機能 ==========

pub async fn handle_admin_login_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let body_str = match get_request_body(req).await {
        Ok(body) => body,
        Err(response) => return Ok(response),
    };

    let request_data: Result<serde_json::Value, _> = serde_json::from_str(&body_str);

    match request_data {
        Ok(data) => {
            if let Some(admin_key) = data.get("admin_key").and_then(|k| k.as_str()) {
                let store = api_key_store.read().await;

                if store.validate_admin_key(admin_key) {
                    let response = serde_json::json!({
                        "success": true,
                        "message": "ログイン成功"
                    });
                    Ok(create_json_response(serde_json::to_string(&response).unwrap(), StatusCode::OK))
                } else {
                    Ok(create_unauthorized_json_response("無効な管理者キーです"))
                }
            } else {
                Ok(create_bad_request_json_response("admin_keyフィールドが必要です"))
            }
        }
        Err(_) => Ok(create_bad_request_json_response("無効なJSONです"))
    }
}

// ========== ヘルパー関数 ==========

async fn get_request_body(req: Request<Body>) -> Result<String, Response<Body>> {
    match hyper::body::to_bytes(req.into_body()).await {
        Ok(body_bytes) => Ok(String::from_utf8_lossy(&body_bytes).to_string()),
        Err(_) => Err(create_bad_request_json_response("リクエストボディの読み取りに失敗しました"))
    }
}

async fn create_new_api_key(key: &str, api_key_store: &SharedApiKeyStore) -> Result<String, ApiKeyError> {
    let mut store = api_key_store.write().await;
    store.add_key(key.to_string())?;
    Ok(format!("APIキー '{}' を作成しました", key))
}

async fn process_url_and_record_usage(target_url: &str, api_key: &str, api_key_store: &SharedApiKeyStore) -> Result<String, String> {
    let normalized_url = normalize_url(target_url);

    match get_html(&normalized_url).await {
        Ok((html_body, final_url)) => {
            let base_url = get_base_url(&final_url);
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);
            let processed_size = processed_html.len() as u64;

            // 使用量を記録
            let mut store = api_key_store.write().await;
            if let Err(e) = store.add_usage(api_key, original_size, processed_size) {
                eprintln!("使用量記録エラー: {}", e);
            }

            Ok(processed_html)
        }
        Err(e) => Err(format!("URL取得エラー: {}", e))
    }
}

async fn process_url_for_api(target_url: &str, api_key: &str, api_key_store: &SharedApiKeyStore) -> ApiResponse {
    let normalized_url = normalize_url(target_url);

    match get_html(&normalized_url).await {
        Ok((html_body, final_url)) => {
            let base_url = get_base_url(&final_url);
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);
            let processed_size = processed_html.len() as u64;

            // 使用量を記録
            let mut store = api_key_store.write().await;
            if let Err(e) = store.add_usage(api_key, original_size, processed_size) {
                eprintln!("使用量記録エラー: {}", e);
            }

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
        Err(e) => create_api_error_response(&e, Some(&normalized_url))
    }
}

fn create_api_error_response(error_msg: &str, original_url: Option<&str>) -> ApiResponse {
    ApiResponse {
        success: false,
        data: None,
        error: Some(error_msg.to_string()),
        original_url: original_url.map(|s| s.to_string()),
        processed_at: chrono::Utc::now().to_rfc3339(),
        original_size_bytes: None,
        processed_size_bytes: None,
    }
}
