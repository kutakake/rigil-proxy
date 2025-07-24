mod api_key;
mod api_types;
mod html_parser;
mod web_ui;
mod handlers;

use api_key::{ApiKeyStore, SharedApiKeyStore};
use web_ui::{get_api_docs_html, get_home_page_html, get_admin_page_html};
use handlers::{
    handle_proxy_request, handle_api_get_request, create_html_response,
    handle_create_key_request, handle_list_keys_request, handle_delete_key_request,
    handle_statistics_request, handle_admin_login_request
};

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

    // デフォルトAPIキーを追加（もし存在しない場合）
    {
        let mut store = api_key_store.write().await;
        if store.list_keys().is_empty() {
            store.add_key("default-api-key".to_string());
            println!("デフォルトAPIキー 'default-api-key' を作成しました");
        }
    }

    println!("管理者キー: {}", ApiKeyStore::get_admin_key());

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
            Ok(create_html_response(html.to_string()))
        }
        (&Method::GET, "/api/docs") => {
            let docs_html = get_api_docs_html();
            Ok(create_html_response(docs_html.to_string()))
        }
        (&Method::GET, "/admin") => {
            let admin_html = get_admin_page_html();
            Ok(create_html_response(admin_html.to_string()))
        }
        (&Method::GET, "/proxy") => {
            handle_proxy_request(req, api_key_store).await
        }
        (&Method::GET, "/api/process") => {
            handle_api_get_request(req, api_key_store).await
        }
        (&Method::POST, "/api/keys/create") => {
            handle_create_key_request(req, api_key_store).await
        }
        (&Method::GET, "/api/keys/list") => {
            handle_list_keys_request(req, api_key_store).await
        }
        (&Method::DELETE, "/api/keys/delete") => {
            handle_delete_key_request(req, api_key_store).await
        }
        (&Method::GET, "/api/statistics") => {
            handle_statistics_request(req, api_key_store).await
        }
        (&Method::POST, "/api/admin/login") => {
            handle_admin_login_request(req, api_key_store).await
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
