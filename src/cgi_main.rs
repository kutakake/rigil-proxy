mod api_key;
mod api_types;
mod html_parser_sync;
mod web_ui;

use api_key::ApiKeyStore;
use api_types::{ApiRequest, ApiResponse};
use html_parser_sync::{get_base_url, get_html_sync, normalize_url, parse_html_to_text};
use web_ui::{get_api_docs_html, get_home_page_html, get_admin_page_html};

use std::collections::HashMap;
use std::env;
use std::io::{self, Read, Write};

fn main() {
    // CGI環境変数を取得
    let request_method = env::var("REQUEST_METHOD").unwrap_or_default();
    let query_string = env::var("QUERY_STRING").unwrap_or_default();
    let path_info = env::var("PATH_INFO").unwrap_or_default();
    let _script_name = env::var("SCRIPT_NAME").unwrap_or_default();
    
    // リクエストのパスを決定
    let path = if !path_info.is_empty() {
        path_info
    } else {
        "/".to_string()
    };

    // APIキーストアを初期化
    let mut api_key_store = ApiKeyStore::load_from_file();

    // リクエストを処理
    match handle_cgi_request(&request_method, &path, &query_string, &mut api_key_store) {
        Ok(response) => {
            print!("{}", response);
            io::stdout().flush().unwrap();
        }
        Err(e) => {
            eprintln!("CGI Error: {}", e);
            print_error_response(&format!("Internal Server Error: {}", e));
        }
    }
}

fn handle_cgi_request(
    method: &str,
    path: &str,
    query_string: &str,
    api_key_store: &mut ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    match (method, path) {
        ("GET", "/") => {
            let html = get_home_page_html();
            Ok(format!("Content-Type: text/html; charset=utf-8\r\n\r\n{}", html))
        }
        ("GET", "/api/docs") => {
            let docs_html = get_api_docs_html();
            Ok(format!("Content-Type: text/html; charset=utf-8\r\n\r\n{}", docs_html))
        }
        ("GET", "/admin") => {
            let admin_html = get_admin_page_html();
            Ok(format!("Content-Type: text/html; charset=utf-8\r\n\r\n{}", admin_html))
        }
        ("GET", "/proxy") => {
            handle_proxy_request(query_string, api_key_store)
        }
        ("GET", "/api/process") => {
            handle_api_get_request(query_string, api_key_store)
        }
        ("POST", "/api/process") => {
            handle_api_post_request(api_key_store)
        }
        ("POST", "/api/keys/create") => {
            handle_create_key_request(api_key_store)
        }
        ("GET", "/api/keys/usage") => {
            handle_usage_request(query_string, api_key_store)
        }
        ("GET", "/api/keys/list") => {
            handle_list_keys_request(query_string, api_key_store)
        }
        ("DELETE", "/api/keys/delete") => {
            handle_delete_key_request(query_string, api_key_store)
        }
        _ => {
            Ok("Status: 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><head><meta charset=\"UTF-8\"></head><body><h1>404 Not Found</h1></body></html>".to_string())
        }
    }
}

fn parse_query_string(query_string: &str) -> HashMap<String, String> {
    url::form_urlencoded::parse(query_string.as_bytes())
        .into_owned()
        .collect()
}

fn handle_proxy_request(
    query_string: &str,
    api_key_store: &mut ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = parse_query_string(query_string);

    if let Some(target_url) = params.get("url") {
        // APIキーのチェック（オプション）
        if let Some(api_key) = params.get("api_key") {
            if !api_key_store.validate_key(api_key) {
                let error_html = "<html><head><meta charset=\"UTF-8\"></head><body><h1>エラー</h1><p>無効なAPIキーです</p></body></html>";
                return Ok(format!("Status: 401 Unauthorized\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}", error_html));
            }
        }

        let normalized_url = normalize_url(target_url);
        let base_url = get_base_url(&normalized_url);

        match get_html_sync(&normalized_url) {
            Ok(html_body) => {
                let original_size = html_body.len() as u64;
                let processed_html = parse_html_to_text(&html_body, &base_url, &normalized_url);

                // APIキーがある場合は使用量を記録
                if let Some(api_key) = params.get("api_key") {
                    api_key_store.add_usage(api_key, original_size);
                    api_key_store.save_to_file();
                }

                Ok(format!("Content-Type: text/html; charset=utf-8\r\n\r\n{}", processed_html))
            }
            Err(e) => {
                let error_html = format!(
                    "<html><head><meta charset=\"UTF-8\"></head><body><h1>エラー</h1><p>URL取得に失敗しました: {}</p><p>対象URL: {}</p></body></html>",
                    e, target_url
                );
                Ok(format!("Content-Type: text/html; charset=utf-8\r\n\r\n{}", error_html))
            }
        }
    } else {
        let error_html = "<html><head><meta charset=\"UTF-8\"></head><body><h1>エラー</h1><p>URLパラメータが必要です</p></body></html>";
        Ok(format!("Content-Type: text/html; charset=utf-8\r\n\r\n{}", error_html))
    }
}

fn handle_api_get_request(
    query_string: &str,
    api_key_store: &mut ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = parse_query_string(query_string);

    if let Some(target_url) = params.get("url") {
        let api_key = params.get("api_key");
        let response = process_url_api(target_url, api_key, api_key_store)?;
        let json_response = serde_json::to_string(&response)?;

        Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", json_response))
    } else {
        let error_response = ApiResponse {
            success: false,
            data: None,
            error: Some("URLパラメータが必要です".to_string()),
            original_url: None,
            processed_at: chrono::Utc::now().to_rfc3339(),
            original_size_bytes: None,
            processed_size_bytes: None,
        };
        let json_response = serde_json::to_string(&error_response)?;
        Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", json_response))
    }
}

fn handle_api_post_request(
    api_key_store: &mut ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    // 標準入力からPOSTデータを読み取り
    let mut stdin = io::stdin();
    let mut buffer = String::new();
    stdin.read_to_string(&mut buffer)?;

    // ヘッダーからAPIキーを取得する場合も対応
    let api_key_from_header = env::var("HTTP_X_API_KEY").ok();

    match serde_json::from_str::<ApiRequest>(&buffer) {
        Ok(api_req) => {
            let response = process_url_api(&api_req.url, api_key_from_header.as_ref(), api_key_store)?;
            let json_response = serde_json::to_string(&response)?;
            Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", json_response))
        }
        Err(_) => {
            let error_response = ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid JSON format".to_string()),
                original_url: None,
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            };
            let json_response = serde_json::to_string(&error_response)?;
            Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", json_response))
        }
    }
}

fn handle_create_key_request(
    api_key_store: &mut ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    // 管理者キーのチェック
    let admin_key = env::var("HTTP_X_ADMIN_KEY").unwrap_or_default();
    if admin_key != "admin_rigil_proxy_master_key_2024" {
        return Ok("Status: 401 Unauthorized\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"error\":\"Invalid admin key\"}".to_string());
    }

    // 新しいAPIキーを生成
    let new_key = format!("api_key_{}", chrono::Utc::now().timestamp());
    match api_key_store.add_key(&admin_key, new_key.clone()) {
        Ok(_) => {
            let response = serde_json::json!({
                "success": true,
                "api_key": new_key,
                "message": "API key created successfully"
            });
            Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "error": e
            });
            Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", response))
        }
    }
}

fn handle_usage_request(
    query_string: &str,
    api_key_store: &ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = parse_query_string(query_string);
    
    if let Some(api_key) = params.get("api_key") {
        if let Some(usage_bytes) = api_key_store.get_usage(api_key) {
            let usage_response = serde_json::json!({
                "api_key": api_key,
                "total_bytes_processed": usage_bytes,
                "last_checked": chrono::Utc::now().to_rfc3339()
            });
            let response = serde_json::to_string(&usage_response)?;
            Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", response))
        } else {
            Ok("Status: 404 Not Found\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"error\":\"API key not found\"}".to_string())
        }
    } else {
        Ok("Status: 400 Bad Request\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"error\":\"API key parameter required\"}".to_string())
    }
}

fn handle_list_keys_request(
    query_string: &str,
    api_key_store: &ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = parse_query_string(query_string);
    let admin_key = params.get("admin_key").map(|s| s.as_str()).unwrap_or("");
    
    if admin_key != "admin_rigil_proxy_master_key_2024" {
        return Ok("Status: 401 Unauthorized\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"error\":\"Invalid admin key\"}".to_string());
    }

    match api_key_store.list_keys(&admin_key) {
        Ok(keys) => {
            let response = serde_json::to_string(&keys)?;
            Ok(format!("Content-Type: application/json; charset=utf-8\r\n\r\n{}", response))
        }
        Err(e) => {
            Ok(format!("Status: 401 Unauthorized\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{{\"error\":\"{}\"}}", e))
        }
    }
}

fn handle_delete_key_request(
    query_string: &str,
    api_key_store: &mut ApiKeyStore,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = parse_query_string(query_string);
    let admin_key = params.get("admin_key").map(|s| s.as_str()).unwrap_or("");
    
    if admin_key != "admin_rigil_proxy_master_key_2024" {
        return Ok("Status: 401 Unauthorized\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"error\":\"Invalid admin key\"}".to_string());
    }

    if let Some(api_key) = params.get("api_key") {
        match api_key_store.remove_key(&admin_key, api_key) {
            Ok(_) => {
                Ok("Content-Type: application/json; charset=utf-8\r\n\r\n{\"success\":true,\"message\":\"Key deleted successfully\"}".to_string())
            }
            Err(e) => {
                Ok(format!("Status: 500 Internal Server Error\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{{\"error\":\"{}\"}}", e))
            }
        }
    } else {
        Ok("Status: 400 Bad Request\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"error\":\"API key parameter required\"}".to_string())
    }
}

fn process_url_api(
    target_url: &str,
    api_key: Option<&String>,
    api_key_store: &mut ApiKeyStore,
) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    // APIキーのチェック
    if let Some(key) = api_key {
        if !api_key_store.validate_key(key) {
            return Ok(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid API key".to_string()),
                original_url: Some(target_url.to_string()),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            });
        }
    }

    let normalized_url = normalize_url(target_url);
    let base_url = get_base_url(&normalized_url);

    match get_html_sync(&normalized_url) {
        Ok(html_body) => {
            let original_size = html_body.len() as u64;
            let processed_text = parse_html_to_text(&html_body, &base_url, &normalized_url);
            let processed_size = processed_text.len() as u64;

            // APIキーがある場合は使用量を記録
            if let Some(key) = api_key {
                api_key_store.add_usage(key, original_size);
                api_key_store.save_to_file();
            }

            Ok(ApiResponse {
                success: true,
                data: Some(processed_text),
                error: None,
                original_url: Some(normalized_url),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: Some(original_size),
                processed_size_bytes: Some(processed_size),
            })
        }
        Err(e) => {
            Ok(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to fetch URL: {}", e)),
                original_url: Some(target_url.to_string()),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            })
        }
    }
}

fn print_error_response(error_message: &str) {
    print!("Status: 500 Internal Server Error\r\n");
    print!("Content-Type: text/html; charset=utf-8\r\n\r\n");
    print!("<html><head><meta charset=\"UTF-8\"></head><body><h1>Internal Server Error</h1><p>{}</p></body></html>", error_message);
} 