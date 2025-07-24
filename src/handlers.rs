use crate::api_key::SharedApiKeyStore;
use crate::api_types::{ApiRequest, ApiResponse, CreateKeyRequest, UsageResponse};
use crate::html_parser::{get_base_url, get_html, normalize_url, parse_html_to_text};

use hyper::{Body, Request, Response, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;

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

// エラーページ生成
pub fn create_error_page(title: &str, message: &str) -> String {
    format!(
        r#"<html><head><meta charset="UTF-8"></head><body>
            <h1>{}</h1>
            <p>{}</p>
            <p><a href="/">ホーム画面に戻る</a></p>
        </body></html>"#,
        title, message
    )
}

// クエリパラメータ解析
pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

// APIキー検証とエラーレスポンス
pub async fn validate_api_key_or_error_html(
    params: &HashMap<String, String>,
    api_key_store: &SharedApiKeyStore,
) -> Result<String, Response<Body>> {
    let api_key = match params.get("api_key") {
        Some(key) => key,
        None => {
            let error_html = create_error_page("認証エラー", "APIキーが必要です。<br>ホーム画面でAPIキーを設定してください。");
            let mut response = create_html_response(error_html);
            *response.status_mut() = StatusCode::UNAUTHORIZED;
            return Err(response);
        }
    };

    let store = api_key_store.read().await;
    if !store.validate_key(api_key) {
        drop(store);
        let error_html = create_error_page("認証エラー", "無効なAPIキーです。<br>ホーム画面で正しいAPIキーを設定してください。");
        let mut response = create_html_response(error_html);
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        return Err(response);
    }
    drop(store);
    
    Ok(api_key.clone())
}

pub async fn validate_api_key_or_error_json(
    params: &HashMap<String, String>,
    api_key_store: &SharedApiKeyStore,
    url: &str,
) -> Result<String, Response<Body>> {
    let api_key = match params.get("api_key") {
        Some(key) => key,
        None => {
            let error_response = ApiResponse {
                success: false,
                data: None,
                error: Some("APIキーが必要です".to_string()),
                original_url: Some(url.to_string()),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            return Err(create_json_response(json_response, StatusCode::UNAUTHORIZED));
        }
    };

    let store = api_key_store.read().await;
    if !store.validate_key(api_key) {
        drop(store);
        let error_response = ApiResponse {
            success: false,
            data: None,
            error: Some("無効なAPIキーです".to_string()),
            original_url: Some(url.to_string()),
            processed_at: chrono::Utc::now().to_rfc3339(),
            original_size_bytes: None,
            processed_size_bytes: None,
        };
        let json_response = serde_json::to_string(&error_response).unwrap();
        return Err(create_json_response(json_response, StatusCode::UNAUTHORIZED));
    }
    drop(store);
    
    Ok(api_key.clone())
}

pub fn validate_api_key_from_header_or_error(
    headers: &hyper::HeaderMap,
    url: &str,
) -> Result<String, Response<Body>> {
    match headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
        Some(key) => Ok(key.to_string()),
        None => {
            let error_response = ApiResponse {
                success: false,
                data: None,
                error: Some("APIキーが必要です（X-API-Keyヘッダーで指定してください）".to_string()),
                original_url: Some(url.to_string()),
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            Err(create_json_response(json_response, StatusCode::UNAUTHORIZED))
        }
    }
}

// プロキシリクエストハンドラー
pub async fn handle_proxy_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    let target_url = match params.get("url") {
        Some(url) => url,
        None => {
            let error_html = create_error_page("パラメータエラー", "URLパラメータが必要です。<br>ホーム画面から正しくアクセスしてください。");
            let mut response = create_html_response(error_html);
            *response.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(response);
        }
    };

    let api_key = match validate_api_key_or_error_html(&params, &api_key_store).await {
        Ok(key) => key,
        Err(response) => return Ok(response),
    };

    let normalized_url = normalize_url(target_url);

    match get_html(&normalized_url).await {
        Ok((html_body, final_url)) => {
            let base_url = get_base_url(&final_url);
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);

            // 使用量を記録
            let processed_size = processed_html.len() as u64;
            let mut store = api_key_store.write().await;
            store.add_usage(&api_key, original_size, processed_size);
            drop(store);

            Ok(create_html_response(processed_html))
        }
        Err(e) => {
            let error_html = format!(
                r#"<html><head><meta charset="UTF-8"></head><body>
                    <h1>エラー</h1>
                    <p>URL取得に失敗しました: {}</p>
                    <p>対象URL: {}</p>
                    <p><a href="/">ホーム画面に戻る</a></p>
                </body></html>"#,
                e, target_url
            );
            Ok(create_html_response(error_html))
        }
    }
}

// API GET リクエストハンドラー
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

    let api_key = match validate_api_key_or_error_json(&params, &api_key_store, target_url).await {
        Ok(key) => key,
        Err(response) => return Ok(response),
    };

    let response = process_url_api(target_url, &api_key, api_key_store).await;
    let json_response = serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"success":false,"data":null,"error":"JSON serialization error","original_url":null,"processed_at":""}"#.to_string()
    });

    Ok(create_json_response(json_response, StatusCode::OK))
}

// API POST リクエストハンドラー
pub async fn handle_api_post_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    // ヘッダーを先に取得
    let headers = req.headers().clone();
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes);

    match serde_json::from_str::<ApiRequest>(&body_str) {
        Ok(api_req) => {
            let api_key = match validate_api_key_from_header_or_error(&headers, &api_req.url) {
                Ok(key) => key,
                Err(response) => return Ok(response),
            };

            let response = process_url_api(&api_req.url, &api_key, api_key_store).await;
            let json_response = serde_json::to_string(&response).unwrap_or_else(|_| {
                r#"{"success":false,"data":null,"error":"JSON serialization error","original_url":null,"processed_at":""}"#.to_string()
            });

            Ok(create_json_response(json_response, StatusCode::OK))
        }
        Err(_) => {
            let error_response = ApiResponse {
                success: false,
                data: None,
                error: Some("無効なJSONリクエストです".to_string()),
                original_url: None,
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
        }
    }
}

// APIキー作成ハンドラー
pub async fn handle_create_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes);

    match serde_json::from_str::<CreateKeyRequest>(&body_str) {
        Ok(create_req) => {
            let mut store = api_key_store.write().await;
            match store.add_key(&create_req.admin_key, create_req.key.clone()) {
                Ok(()) => {
                    drop(store);
                    let usage_response = UsageResponse {
                        success: true,
                        key: Some(create_req.key),
                        total_bytes_processed: Some(0),
                        keys: None,
                        error: None,
                    };
                    let json_response = serde_json::to_string(&usage_response).unwrap();
                    Ok(create_json_response(json_response, StatusCode::OK))
                }
                Err(err_msg) => {
                    drop(store);
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
        Err(_) => {
            let error_response = UsageResponse {
                success: false,
                key: None,
                total_bytes_processed: None,
                keys: None,
                error: Some("無効なJSONリクエスト".to_string()),
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
        }
    }
}

// APIキー使用量取得ハンドラー
pub async fn handle_usage_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if let Some(api_key) = params.get("api_key") {
        let store = api_key_store.read().await;
        if let Some(usage) = store.get_usage(api_key) {
            let usage_response = UsageResponse {
                success: true,
                key: Some(api_key.clone()),
                total_bytes_processed: Some(usage),
                keys: None,
                error: None,
            };
            let json_response = serde_json::to_string(&usage_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::OK))
        } else {
            let error_response = UsageResponse {
                success: false,
                key: None,
                total_bytes_processed: None,
                keys: None,
                error: Some("無効なAPIキーです".to_string()),
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::UNAUTHORIZED))
        }
    } else {
        let error_response = UsageResponse {
            success: false,
            key: None,
            total_bytes_processed: None,
            keys: None,
            error: Some("api_keyパラメータが必要です".to_string()),
        };
        let json_response = serde_json::to_string(&error_response).unwrap();
        Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
    }
}

// APIキー一覧取得ハンドラー
pub async fn handle_list_keys_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if let Some(admin_key) = params.get("admin_key") {
        let store = api_key_store.read().await;
        match store.list_keys(admin_key) {
            Ok(keys) => {
                let usage_response = UsageResponse {
                    success: true,
                    key: None,
                    total_bytes_processed: None,
                    keys: Some(keys),
                    error: None,
                };
                let json_response = serde_json::to_string(&usage_response).unwrap();
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
    } else {
        let error_response = UsageResponse {
            success: false,
            key: None,
            total_bytes_processed: None,
            keys: None,
            error: Some("admin_keyパラメータが必要です".to_string()),
        };
        let json_response = serde_json::to_string(&error_response).unwrap();
        Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
    }
}

// APIキー削除ハンドラー
pub async fn handle_delete_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params = parse_query_params(query);

    if let Some(admin_key) = params.get("admin_key") {
        if let Some(key_to_delete) = params.get("key") {
            let mut store = api_key_store.write().await;
            match store.remove_key(admin_key, key_to_delete) {
                Ok(()) => {
                    let success_response = UsageResponse {
                        success: true,
                        key: Some(key_to_delete.clone()),
                        total_bytes_processed: None,
                        keys: None,
                        error: None,
                    };
                    let json_response = serde_json::to_string(&success_response).unwrap();
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
        } else {
            let error_response = UsageResponse {
                success: false,
                key: None,
                total_bytes_processed: None,
                keys: None,
                error: Some("keyパラメータが必要です".to_string()),
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
        }
    } else {
        let error_response = UsageResponse {
            success: false,
            key: None,
            total_bytes_processed: None,
            keys: None,
            error: Some("admin_keyパラメータが必要です".to_string()),
        };
        let json_response = serde_json::to_string(&error_response).unwrap();
        Ok(create_json_response(json_response, StatusCode::BAD_REQUEST))
    }
}

// URL処理API
async fn process_url_api(target_url: &str, api_key: &str, api_key_store: SharedApiKeyStore) -> ApiResponse {
    let normalized_url = normalize_url(target_url);

    // APIキーの検証
    let store = api_key_store.read().await;
    if !store.validate_key(api_key) {
        drop(store);
        return ApiResponse {
            success: false,
            data: None,
            error: Some("無効なAPIキーです".to_string()),
            original_url: Some(normalized_url),
            processed_at: chrono::Utc::now().to_rfc3339(),
            original_size_bytes: None,
            processed_size_bytes: None,
        };
    }
    drop(store);

    match get_html(&normalized_url).await {
        Ok((html_body, final_url)) => {
            let base_url = get_base_url(&final_url);
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);
            let processed_size = processed_html.len() as u64;

            // 使用量を記録
            let mut store = api_key_store.write().await;
            store.add_usage(api_key, original_size, processed_size);
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
    }
} 