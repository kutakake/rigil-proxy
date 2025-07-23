mod api_key;
mod api_types;
mod html_parser;
mod web_ui;

use api_key::{ApiKeyStore, SharedApiKeyStore};
use api_types::{ApiRequest, ApiResponse, CreateKeyRequest, UsageResponse};
use html_parser::{get_base_url, get_html, normalize_url, parse_html_to_text};
use web_ui::{get_api_docs_html, get_home_page_html, get_admin_page_html};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

const PORT: u16 = 80;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));

    // APIキーストアを初期化
    let api_key_store = Arc::new(RwLock::new(ApiKeyStore::load_from_file()));

    let api_key_store_clone = api_key_store.clone();
    let make_svc = make_service_fn(move |_conn| {
        let store = api_key_store_clone.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| handle_request(req, store.clone())))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Rigil Proxy server running on port {}", PORT);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let html = get_home_page_html();
            let mut response = Response::new(Body::from(html));
            response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
            Ok(response)
        }
        (&Method::GET, "/api/docs") => {
            let docs_html = get_api_docs_html();
            let mut response = Response::new(Body::from(docs_html));
            response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
            Ok(response)
        }
        (&Method::GET, "/admin") => {
            let admin_html = get_admin_page_html();
            let mut response = Response::new(Body::from(admin_html));
            response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
            Ok(response)
        }
        (&Method::GET, "/proxy") => {
            handle_proxy_request(req, api_key_store).await
        }
        (&Method::GET, "/api/process") => {
            handle_api_get_request(req, api_key_store).await
        }
        (&Method::POST, "/api/process") => {
            handle_api_post_request(req, api_key_store).await
        }
        (&Method::POST, "/api/keys/create") => {
            handle_create_key_request(req, api_key_store).await
        }
        (&Method::GET, "/api/keys/usage") => {
            handle_usage_request(req, api_key_store).await
        }
        (&Method::GET, "/api/keys/list") => {
            handle_list_keys_request(req, api_key_store).await
        }
        (&Method::DELETE, "/api/keys/delete") => {
            handle_delete_key_request(req, api_key_store).await
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn handle_proxy_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    if let Some(target_url) = params.get("url") {
        // APIキーのチェック（オプション）
        if let Some(api_key) = params.get("api_key") {
            let store = api_key_store.read().await;
            if !store.validate_key(api_key) {
                drop(store);
                let error_html = "<html><head><meta charset=\"UTF-8\"></head><body><h1>エラー</h1><p>無効なAPIキーです</p></body></html>";
                let mut response = Response::new(Body::from(error_html));
                *response.status_mut() = StatusCode::UNAUTHORIZED;
                response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
                return Ok(response);
            }
            drop(store);
        }

        let normalized_url = normalize_url(target_url);

        match get_html(&normalized_url).await {
            Ok((html_body, final_url)) => {
                // リダイレクト後の最終URLを使用してbase_urlを計算
                let base_url = get_base_url(&final_url);
                let original_size = html_body.len() as u64;
                let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);

                // APIキーがある場合は使用量を記録
                if let Some(api_key) = params.get("api_key") {
                    let mut store = api_key_store.write().await;
                    store.add_usage(api_key, original_size);
                    drop(store);
                }

                let mut response = Response::new(Body::from(processed_html));
                response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
                Ok(response)
            }
            Err(e) => {
                let error_html = format!(
                    "<html><head><meta charset=\"UTF-8\"></head><body><h1>エラー</h1><p>URL取得に失敗しました: {}</p><p>対象URL: {}</p></body></html>",
                    e, target_url
                );
                let mut response = Response::new(Body::from(error_html));
                response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
                Ok(response)
            }
        }
    } else {
        let error_html = "<html><head><meta charset=\"UTF-8\"></head><body><h1>エラー</h1><p>URLパラメータが必要です</p></body></html>";
        let mut response = Response::new(Body::from(error_html));
        response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
        Ok(response)
    }
}

async fn handle_api_get_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    if let Some(target_url) = params.get("url") {
        let api_key = params.get("api_key");
        let response = process_url_api(target_url, api_key, api_key_store.clone()).await;
        let json_response = serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"success":false,"data":null,"error":"JSON serialization error","original_url":null,"processed_at":""}"#.to_string()
        });

        let mut resp = Response::new(Body::from(json_response));
        resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
        Ok(resp)
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
        let json_response = serde_json::to_string(&error_response).unwrap();
        let mut resp = Response::new(Body::from(json_response));
        resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
        Ok(resp)
    }
}

async fn handle_api_post_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    // ヘッダーからAPIキーを取得する場合も対応
    let api_key_from_header = req.headers().get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes);

    match serde_json::from_str::<ApiRequest>(&body_str) {
        Ok(api_req) => {
            let response = process_url_api(&api_req.url, api_key_from_header.as_ref(), api_key_store.clone()).await;
            let json_response = serde_json::to_string(&response).unwrap_or_else(|_| {
                r#"{"success":false,"data":null,"error":"JSON serialization error","original_url":null,"processed_at":""}"#.to_string()
            });

            let mut resp = Response::new(Body::from(json_response));
            resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
            Ok(resp)
        }
        Err(_) => {
            let error_response = ApiResponse {
                success: false,
                data: None,
                error: Some("無効なJSONリクエスト".to_string()),
                original_url: None,
                processed_at: chrono::Utc::now().to_rfc3339(),
                original_size_bytes: None,
                processed_size_bytes: None,
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            let mut resp = Response::new(Body::from(json_response));
            resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
            Ok(resp)
        }
    }
}

async fn handle_create_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
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
                    let mut resp = Response::new(Body::from(json_response));
                    resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
                    Ok(resp)
                },
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
                    let mut resp = Response::new(Body::from(json_response));
                    *resp.status_mut() = StatusCode::UNAUTHORIZED;
                    resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
                    Ok(resp)
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
            let mut resp = Response::new(Body::from(json_response));
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
            Ok(resp)
        }
    }
}

async fn handle_usage_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

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
            let mut resp = Response::new(Body::from(json_response));
            resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
            Ok(resp)
        } else {
            let error_response = UsageResponse {
                success: false,
                key: None,
                total_bytes_processed: None,
                keys: None,
                error: Some("無効なAPIキーです".to_string()),
            };
            let json_response = serde_json::to_string(&error_response).unwrap();
            let mut resp = Response::new(Body::from(json_response));
            *resp.status_mut() = StatusCode::UNAUTHORIZED;
            resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
            Ok(resp)
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
        let mut resp = Response::new(Body::from(json_response));
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
        Ok(resp)
    }
}

async fn handle_list_keys_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

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
                let mut resp = Response::new(Body::from(json_response));
                resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
                Ok(resp)
            },
            Err(err_msg) => {
                let error_response = UsageResponse {
                    success: false,
                    key: None,
                    total_bytes_processed: None,
                    keys: None,
                    error: Some(err_msg),
                };
                let json_response = serde_json::to_string(&error_response).unwrap();
                let mut resp = Response::new(Body::from(json_response));
                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
                Ok(resp)
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
        let mut resp = Response::new(Body::from(json_response));
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
        Ok(resp)
    }
}

async fn handle_delete_key_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

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
                    let mut resp = Response::new(Body::from(json_response));
                    resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
                    Ok(resp)
                },
                Err(err_msg) => {
                    let error_response = UsageResponse {
                        success: false,
                        key: None,
                        total_bytes_processed: None,
                        keys: None,
                        error: Some(err_msg),
                    };
                    let json_response = serde_json::to_string(&error_response).unwrap();
                    let mut resp = Response::new(Body::from(json_response));
                    *resp.status_mut() = StatusCode::UNAUTHORIZED;
                    resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
                    Ok(resp)
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
            let mut resp = Response::new(Body::from(json_response));
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
            Ok(resp)
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
        let mut resp = Response::new(Body::from(json_response));
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp.headers_mut().insert("content-type", "application/json; charset=utf-8".parse().unwrap());
        Ok(resp)
    }
}

async fn process_url_api(target_url: &str, api_key: Option<&String>, api_key_store: SharedApiKeyStore) -> ApiResponse {
    let normalized_url = normalize_url(target_url);

    // APIキーの検証
    if let Some(key) = api_key {
        let store = api_key_store.read().await;
        if !store.validate_key(key) {
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
    }

    match get_html(&normalized_url).await {
        Ok((html_body, final_url)) => {
            // リダイレクト後の最終URLを使用してbase_urlを計算
            let base_url = get_base_url(&final_url);
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &final_url);
            let processed_size = processed_html.len() as u64;

            // APIキーがある場合は使用量を記録
            if let Some(key) = api_key {
                let mut store = api_key_store.write().await;
                store.add_usage(key, original_size);
                drop(store);
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
