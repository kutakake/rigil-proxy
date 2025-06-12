use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use url::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ApiKeyData {
    key: String,
    total_bytes_processed: u64,
    created_at: String,
    last_used: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ApiKeyStore {
    keys: HashMap<String, ApiKeyData>,
}

impl ApiKeyStore {
    fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    fn load_from_file() -> Self {
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

    fn save_to_file(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write("api_keys.json", content);
        }
    }

    fn add_key(&mut self, key: String) {
        let api_key_data = ApiKeyData {
            key: key.clone(),
            total_bytes_processed: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };
        self.keys.insert(key, api_key_data);
        self.save_to_file();
    }

    fn validate_key(&self, key: &str) -> bool {
        self.keys.contains_key(key)
    }

    fn add_usage(&mut self, key: &str, bytes: u64) {
        if let Some(api_key_data) = self.keys.get_mut(key) {
            api_key_data.total_bytes_processed += bytes;
            api_key_data.last_used = Some(chrono::Utc::now().to_rfc3339());
            self.save_to_file();
        }
    }

    fn get_usage(&self, key: &str) -> Option<u64> {
        self.keys.get(key).map(|data| data.total_bytes_processed)
    }

    fn list_keys(&self) -> Vec<ApiKeyData> {
        self.keys.values().cloned().collect()
    }
}

type SharedApiKeyStore = Arc<RwLock<ApiKeyStore>>;

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    success: bool,
    data: Option<String>,
    error: Option<String>,
    original_url: Option<String>,
    processed_at: String,
    original_size_bytes: Option<u64>,
    processed_size_bytes: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct ApiRequest {
    url: String,
    format: Option<String>, // "html" or "json"
}

#[derive(Serialize, Deserialize)]
struct CreateKeyRequest {
    key: String,
}

#[derive(Serialize, Deserialize)]
struct UsageResponse {
    success: bool,
    key: Option<String>,
    total_bytes_processed: Option<u64>,
    keys: Option<Vec<ApiKeyData>>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 48588));

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

    println!("Rigil Proxy server running on http://{}", addr);
    println!("Web UI: http://0.0.0.0:8080");
    println!("API Documentation: http://0.0.0.0:8080/api/docs");

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>, api_key_store: SharedApiKeyStore) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rigil Proxy</title>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 40px;
            background-color: #fafafa;
            color: #333;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 4px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        input[type="text"] {
            width: 70%;
            padding: 8px 12px;
            font-size: 14px;
            border: 1px solid #ccc;
            border-radius: 2px;
            font-family: inherit;
        }
        button {
            padding: 8px 16px;
            font-size: 14px;
            margin-left: 10px;
            background-color: #f8f9fa;
            color: #333;
            border: 1px solid #ccc;
            border-radius: 2px;
            cursor: pointer;
            font-family: inherit;
        }
        button:hover {
            background-color: #e9ecef;
            border-color: #adb5bd;
        }
        .api-section {
            margin-top: 30px;
            padding: 20px;
            background-color: #f8f9fa;
            border-radius: 2px;
            border: 1px solid #e9ecef;
        }
        .endpoint {
            margin: 10px 0;
            font-family: 'Courier New', monospace;
            background: #fff;
            padding: 8px;
            border-radius: 2px;
            border: 1px solid #e9ecef;
            font-size: 13px;
        }
        h1 {
            color: #333;
            border-bottom: 1px solid #e9ecef;
            padding-bottom: 10px;
            font-weight: normal;
        }
        h2 {
            color: #555;
            font-weight: normal;
            font-size: 18px;
        }
        h3 {
            color: #666;
            font-weight: normal;
            font-size: 16px;
        }
        a {
            color: #666;
            text-decoration: underline;
        }
        a:hover {
            color: #333;
        }
        .api-key-section {
            margin-top: 30px;
            padding: 20px;
            background-color: #f0f8ff;
            border-radius: 2px;
            border: 1px solid #d1ecf1;
        }
        .api-key-form {
            display: flex;
            align-items: center;
            margin: 10px 0;
        }
        .api-key-form input {
            width: 250px;
            margin-right: 10px;
        }
        .generate-btn {
            background-color: #007bff;
            color: white;
            border: 1px solid #007bff;
            margin-left: 5px;
        }
        .generate-btn:hover {
            background-color: #0056b3;
            border-color: #004085;
        }
        .result-box {
            margin: 15px 0;
            padding: 10px;
            background-color: #fff;
            border: 1px solid #e9ecef;
            border-radius: 2px;
            min-height: 20px;
        }
        .success {
            color: #155724;
            background-color: #d4edda;
            border-color: #c3e6cb;
        }
        .error {
            color: #721c24;
            background-color: #f8d7da;
            border-color: #f5c6cb;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Rigil Proxy - HTML軽量化プロキシ</h1>
        <p>URLを入力してHTML軽量化を試してください：</p>
        <form action="/proxy" method="get">
            <input type="text" name="url" placeholder="https://example.com" required>
            <button type="submit">軽量化</button>
        </form>

        <div class="api-key-section">
            <h2>APIキー管理</h2>
            <p>テスト用のAPIキーを作成・管理できます：</p>
            
            <div class="api-key-form">
                <input type="text" id="apiKeyInput" placeholder="APIキー名を入力（例：my-test-key）" value="">
                <button type="button" onclick="createApiKey()">APIキー作成</button>
                <button type="button" class="generate-btn" onclick="generateRandomKey()">ランダム生成</button>
            </div>
            
            <div id="resultBox" class="result-box"></div>
            
            <div style="margin-top: 15px;">
                <button type="button" onclick="listApiKeys()">全APIキー表示</button>
                <button type="button" onclick="checkUsage()">使用量確認</button>
            </div>
            
            <div id="apiKeysList" class="result-box" style="margin-top: 10px;"></div>
        </div>

        <script>
            function generateRandomKey() {
                const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
                let result = 'key-';
                for (let i = 0; i < 12; i++) {
                    result += chars.charAt(Math.floor(Math.random() * chars.length));
                }
                document.getElementById('apiKeyInput').value = result;
            }

            async function createApiKey() {
                const keyInput = document.getElementById('apiKeyInput');
                const resultBox = document.getElementById('resultBox');
                const apiKey = keyInput.value.trim();
                
                if (!apiKey) {
                    resultBox.className = 'result-box error';
                    resultBox.textContent = 'APIキー名を入力してください';
                    return;
                }
                
                try {
                    const response = await fetch('/api/keys/create', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({ key: apiKey })
                    });
                    
                    const data = await response.json();
                    
                    if (data.success) {
                        resultBox.className = 'result-box success';
                        resultBox.textContent = `APIキー "${apiKey}" を作成しました！`;
                        keyInput.value = '';
                    } else {
                        resultBox.className = 'result-box error';
                        resultBox.textContent = `エラー: ${data.error || '不明なエラー'}`;
                    }
                } catch (error) {
                    resultBox.className = 'result-box error';
                    resultBox.textContent = `ネットワークエラー: ${error.message}`;
                }
            }

            async function listApiKeys() {
                const listBox = document.getElementById('apiKeysList');
                
                try {
                    const response = await fetch('/api/keys/list');
                    const data = await response.json();
                    
                    if (data.success && data.keys) {
                        if (data.keys.length === 0) {
                            listBox.textContent = 'APIキーが登録されていません';
                        } else {
                            listBox.innerHTML = '<h4>登録済みAPIキー:</h4>' + 
                                data.keys.map(key => 
                                    `<div style="margin: 5px 0; padding: 5px; border: 1px solid #ddd; border-radius: 2px;">
                                        <strong>キー:</strong> ${key.key}<br>
                                        <strong>使用量:</strong> ${key.total_bytes_processed.toLocaleString()} bytes<br>
                                        <strong>作成日:</strong> ${new Date(key.created_at).toLocaleString()}<br>
                                        <strong>最終使用:</strong> ${key.last_used ? new Date(key.last_used).toLocaleString() : '未使用'}
                                    </div>`
                                ).join('');
                        }
                    } else {
                        listBox.textContent = `エラー: ${data.error || '不明なエラー'}`;
                    }
                } catch (error) {
                    listBox.textContent = `ネットワークエラー: ${error.message}`;
                }
            }

            async function checkUsage() {
                const keyInput = document.getElementById('apiKeyInput');
                const resultBox = document.getElementById('resultBox');
                const apiKey = keyInput.value.trim();
                
                if (!apiKey) {
                    resultBox.className = 'result-box error';
                    resultBox.textContent = '使用量を確認したいAPIキー名を入力してください';
                    return;
                }
                
                try {
                    const response = await fetch(`/api/keys/usage?api_key=${encodeURIComponent(apiKey)}`);
                    const data = await response.json();
                    
                    if (data.success) {
                        resultBox.className = 'result-box success';
                        resultBox.textContent = `APIキー "${apiKey}" の使用量: ${data.total_bytes_processed.toLocaleString()} bytes`;
                    } else {
                        resultBox.className = 'result-box error';
                        resultBox.textContent = `エラー: ${data.error || '不明なエラー'}`;
                    }
                } catch (error) {
                    resultBox.className = 'result-box error';
                    resultBox.textContent = `ネットワークエラー: ${error.message}`;
                }
            }
        </script>

        <div class="api-section">
            <h2>API エンドポイント</h2>
            <p>プログラムからアクセスする場合は以下のAPIを使用してください：</p>

            <h3>HTML軽量化 (GET)</h3>
            <div class="endpoint">GET /proxy?url=https://example.com&api_key=your_api_key</div>
            <p>軽量化されたHTMLを直接返します。api_keyはオプションです。</p>

            <h3>JSON API (GET)</h3>
            <div class="endpoint">GET /api/process?url=https://example.com&api_key=your_api_key</div>
            <p>JSON形式で結果を返します。api_keyはオプションです。</p>

            <h3>JSON API (POST)</h3>
            <div class="endpoint">POST /api/process</div>
            <p>リクエストボディ: {"url": "https://example.com", "format": "json"}</p>
            <p>APIキーはX-API-Keyヘッダーで指定可能です。</p>

            <h3>APIキー管理</h3>
            <div class="endpoint">POST /api/keys/create</div>
            <p>新しいAPIキーを作成します。リクエストボディ: {"key": "your_api_key"}</p>
            
            <div class="endpoint">GET /api/keys/usage?api_key=your_api_key</div>
            <p>指定したAPIキーの使用量を取得します。</p>
            
            <div class="endpoint">GET /api/keys/list</div>
            <p>全てのAPIキーと使用量を一覧表示します。</p>

            <p><a href="/api/docs">詳細なAPIドキュメント</a></p>
        </div>
    </div>
</body>
</html>
            "#;
            let mut response = Response::new(Body::from(html));
            response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
            Ok(response)
        }
        (&Method::GET, "/api/docs") => {
            let docs_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rigil Proxy API Documentation</title>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 40px;
            line-height: 1.6;
            color: #333;
            background-color: #fafafa;
        }
        .container {
            max-width: 1000px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 4px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        .endpoint {
            background: #f8f9fa;
            padding: 15px;
            margin: 15px 0;
            border-left: 3px solid #ccc;
            border-radius: 2px;
        }
        .method {
            display: inline-block;
            padding: 4px 8px;
            border-radius: 2px;
            font-weight: bold;
            color: white;
            font-size: 12px;
        }
        .get { background-color: #6c757d; }
        .post { background-color: #495057; }
        code {
            background: #f8f9fa;
            padding: 2px 4px;
            border-radius: 2px;
            font-family: 'Courier New', monospace;
            font-size: 13px;
        }
        pre {
            background: #f8f9fa;
            padding: 15px;
            border-radius: 2px;
            overflow-x: auto;
            border: 1px solid #e9ecef;
            font-family: 'Courier New', monospace;
            font-size: 13px;
        }
        h1 {
            color: #333;
            border-bottom: 1px solid #e9ecef;
            padding-bottom: 10px;
            font-weight: normal;
        }
        h2 {
            color: #555;
            margin-top: 30px;
            font-weight: normal;
        }
        h3 {
            color: #666;
            font-weight: normal;
        }
        a {
            color: #666;
            text-decoration: underline;
        }
        a:hover {
            color: #333;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Rigil Proxy API Documentation</h1>

        <h2>概要</h2>
        <p>Rigil ProxyはHTML軽量化機能を提供するRESTful APIです。Rigil-Browserと同じアルゴリズムを使用してHTMLを軽量化し、不要なJavaScript、CSS、タグを除去します。</p>

        <h2>エンドポイント</h2>

        <div class="endpoint">
            <h3><span class="method get">GET</span> /proxy</h3>
            <p><strong>説明:</strong> 指定されたURLのHTMLを軽量化して返します。</p>
            <p><strong>パラメータ:</strong></p>
            <ul>
                <li><code>url</code> (必須): 軽量化したいWebページのURL</li>
                <li><code>api_key</code> (オプション): APIキー</li>
            </ul>
            <p><strong>レスポンス:</strong> 軽量化されたHTML (Content-Type: text/html)</p>
            <p><strong>例:</strong></p>
            <pre>GET /proxy?url=https://example.com&api_key=your_api_key</pre>
        </div>

        <div class="endpoint">
            <h3><span class="method get">GET</span> /api/process</h3>
            <p><strong>説明:</strong> 指定されたURLのHTMLを軽量化してJSON形式で返します。</p>
            <p><strong>パラメータ:</strong></p>
            <ul>
                <li><code>url</code> (必須): 軽量化したいWebページのURL</li>
                <li><code>api_key</code> (オプション): APIキー</li>
            </ul>
            <p><strong>レスポンス:</strong> JSON形式の結果</p>
            <p><strong>例:</strong></p>
            <pre>GET /api/process?url=https://example.com&api_key=your_api_key

レスポンス:
{
  "success": true,
  "data": "&lt;html&gt;...&lt;/html&gt;",
  "error": null,
  "original_url": "https://example.com",
  "processed_at": "2024-01-01T12:00:00Z",
  "original_size_bytes": 5120,
  "processed_size_bytes": 1024
}</pre>
        </div>

        <div class="endpoint">
            <h3><span class="method post">POST</span> /api/process</h3>
            <p><strong>説明:</strong> JSON形式のリクエストでHTMLを軽量化します。</p>
            <p><strong>Content-Type:</strong> application/json</p>
            <p><strong>ヘッダー:</strong></p>
            <ul>
                <li><code>X-API-Key</code> (オプション): APIキー</li>
            </ul>
            <p><strong>リクエストボディ:</strong></p>
            <pre>{
  "url": "https://example.com",
  "format": "json"  // オプション: "html" または "json"
}</pre>
            <p><strong>レスポンス:</strong> JSON形式の結果</p>
            <p><strong>例:</strong></p>
            <pre>POST /api/process
Content-Type: application/json
X-API-Key: your_api_key

{
  "url": "https://example.com"
}

レスポンス:
{
  "success": true,
  "data": "&lt;html&gt;...&lt;/html&gt;",
  "error": null,
  "original_url": "https://example.com",
  "processed_at": "2024-01-01T12:00:00Z",
  "original_size_bytes": 5120,
  "processed_size_bytes": 1024
}</pre>
        </div>

        <div class="endpoint">
            <h3><span class="method post">POST</span> /api/keys/create</h3>
            <p><strong>説明:</strong> 新しいAPIキーを作成します。</p>
            <p><strong>Content-Type:</strong> application/json</p>
            <p><strong>リクエストボディ:</strong></p>
            <pre>{"key": "your_api_key"}</pre>
            <p><strong>例:</strong></p>
            <pre>POST /api/keys/create
Content-Type: application/json

{"key": "my-unique-api-key"}

レスポンス:
{
  "success": true,
  "key": "my-unique-api-key",
  "total_bytes_processed": 0,
  "keys": null,
  "error": null
}</pre>
        </div>

        <div class="endpoint">
            <h3><span class="method get">GET</span> /api/keys/usage</h3>
            <p><strong>説明:</strong> 指定したAPIキーの使用量を取得します。</p>
            <p><strong>パラメータ:</strong></p>
            <ul>
                <li><code>api_key</code> (必須): 使用量を確認したいAPIキー</li>
            </ul>
            <p><strong>例:</strong></p>
            <pre>GET /api/keys/usage?api_key=my-unique-api-key

レスポンス:
{
  "success": true,
  "key": "my-unique-api-key",
  "total_bytes_processed": 102400,
  "keys": null,
  "error": null
}</pre>
        </div>

        <div class="endpoint">
            <h3><span class="method get">GET</span> /api/keys/list</h3>
            <p><strong>説明:</strong> 全てのAPIキーとその使用量を一覧表示します。</p>
            <p><strong>例:</strong></p>
            <pre>GET /api/keys/list

レスポンス:
{
  "success": true,
  "key": null,
  "total_bytes_processed": null,
  "keys": [
    {
      "key": "my-unique-api-key",
      "total_bytes_processed": 102400,
      "created_at": "2024-01-01T12:00:00Z",
      "last_used": "2024-01-01T14:30:00Z"
    }
  ],
  "error": null
}</pre>
        </div>

        <h2>エラーレスポンス</h2>
        <p>エラーが発生した場合、以下の形式でJSONが返されます：</p>
        <pre>{
  "success": false,
  "data": null,
  "error": "エラーメッセージ",
  "original_url": "https://example.com",
  "processed_at": "2024-01-01T12:00:00Z"
}</pre>

        <h2>軽量化処理の詳細</h2>
        <ul>
            <li><strong>除去されるタグ:</strong> &lt;script&gt;、&lt;style&gt;、その他の不要なタグ</li>
            <li><strong>保持されるタグ:</strong> title、br、h1-h6、b、i、ul、li、ol</li>
            <li><strong>リンク変換:</strong> &lt;a&gt;タグはプロキシ経由のリンクに変換</li>
            <li><strong>URL正規化:</strong> 相対URLは絶対URLに変換</li>
        </ul>

        <p><a href="/">← ホームに戻る</a></p>
    </div>
</body>
</html>
            "#;
            let mut response = Response::new(Body::from(docs_html));
            response.headers_mut().insert("content-type", "text/html; charset=utf-8".parse().unwrap());
            Ok(response)
        }
        (&Method::GET, "/proxy") => {
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
                let base_url = get_base_url(&normalized_url);

                match get_html(&normalized_url).await {
                    Ok(html_body) => {
                        let original_size = html_body.len() as u64;
                        let processed_html = parse_html_to_text(&html_body, &base_url, &normalized_url);

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
        (&Method::GET, "/api/process") => {
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
        (&Method::POST, "/api/process") => {
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
        // APIキー管理エンドポイント
        (&Method::POST, "/api/keys/create") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&body_bytes);

            match serde_json::from_str::<CreateKeyRequest>(&body_str) {
                Ok(create_req) => {
                    let mut store = api_key_store.write().await;
                    store.add_key(create_req.key.clone());
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
        (&Method::GET, "/api/keys/usage") => {
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
        (&Method::GET, "/api/keys/list") => {
            let store = api_key_store.read().await;
            let keys = store.list_keys();
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
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn process_url_api(target_url: &str, api_key: Option<&String>, api_key_store: SharedApiKeyStore) -> ApiResponse {
    let normalized_url = normalize_url(target_url);
    let base_url = get_base_url(&normalized_url);

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
        Ok(html_body) => {
            let original_size = html_body.len() as u64;
            let processed_html = parse_html_to_text(&html_body, &base_url, &normalized_url);
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
                original_url: Some(normalized_url),
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

// URLを正規化する関数（Rigil-Browserと同じ）
fn normalize_url(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    let mut namestring = name.to_string();
    let name_length = namestring.len();
    let check_length = if name_length >= 8 { 8 } else { name_length };
    let first_part: String = namestring.chars().take(check_length).collect();

    if !first_part.contains("http://") && !first_part.contains("https://") {
        namestring = format!("https://{}", namestring);
    }

    namestring
}

// ベースURLを取得する関数（Rigil-Browserと同じ）
fn get_base_url(url: &str) -> String {
    let url_chars: Vec<char> = url.chars().collect();
    let mut base_url = String::new();
    let mut slash_count = 0;

    for &ch in url_chars.iter() {
        base_url.push(ch);
        if ch == '/' {
            slash_count += 1;
            if slash_count == 3 {
                break;
            }
        }
    }

    // パス部分の処理
    if slash_count == 3 && url.len() > base_url.len() {
        let remaining_path = &url[base_url.len()..];
        if let Some(last_slash_pos) = remaining_path.rfind('/') {
            base_url.push_str(&remaining_path[..=last_slash_pos]);
        }
    }

    base_url
}

// 相対URLを絶対URLに変換する関数（Rigil-Browserと同じ）
fn resolve_relative_url(href: &str, base_url: &str, current_url: &str) -> String {
    if href.contains("http") {
        return href.to_string();
    }

    if href.starts_with('/') {
        // 絶対パス（ルートからの相対パス）
        let mut domain_only = String::new();
        let mut slash_count = 0;
        for ch in current_url.chars() {
            domain_only.push(ch);
            if ch == '/' {
                slash_count += 1;
                if slash_count == 3 {
                    domain_only.pop();
                    break;
                }
            }
        }
        format!("{}{}", domain_only, href)
    } else {
        // 相対パス
        if base_url.ends_with('/') {
            format!("{}{}", base_url, href)
        } else {
            format!("{}/{}", base_url, href)
        }
    }
}

// hrefを抽出する関数（Rigil-Browserと同じ）
fn extract_href(tag: &str) -> String {
    let tag_chars: Vec<char> = tag.chars().collect();
    let mut href = String::new();
    let mut i = 1;

    while i < tag_chars.len() {
        if tag_chars[i] == '"' {
            i += 1;
            while i < tag_chars.len() && tag_chars[i] != '"' {
                href.push(tag_chars[i]);
                i += 1;
            }
            break;
        }
        i += 1;
    }

    href
}

// リンクタグを処理する関数（プロキシ用に修正）
fn process_link_tag(tag: &str, contents: &[char], i: &mut usize, base_url: &str, current_url: &str) -> String {
    let href = extract_href(tag);
    if href.is_empty() {
        return String::new();
    }

    let resolved_href = resolve_relative_url(&href, base_url, current_url);

    // リンクテキストを取得するため、</a>まで読み進める
    let mut link_content = String::new();

    while *i < contents.len() {
        if contents[*i] == '<' {
            // 新しいタグの開始をチェック
            let mut peek_tag = String::new();
            let mut peek_i = *i;

            while peek_i < contents.len() && contents[peek_i] != '>' {
                peek_tag.push(contents[peek_i]);
                peek_i += 1;
            }
            if peek_i < contents.len() {
                peek_tag.push(contents[peek_i]);
            }

            if peek_tag.to_lowercase().contains("</a>") {
                // 終了タグが見つかった
                *i = peek_i + 1;
                break;
            } else if peek_tag.starts_with("<a ") || peek_tag == "<a>" {
                // ネストしたaタグ（スキップ）
            }

            // タグをスキップ
            *i = peek_i + 1;
        } else {
            // 通常のテキスト
            link_content.push(contents[*i]);
            *i += 1;
        }
    }

    // リンクテキストが空の場合はURLを使用
    let display_text = if link_content.trim().is_empty() {
        resolved_href.clone()
    } else {
        link_content.trim().to_string()
    };

    // プロキシ経由でリンクを処理するように修正
    format!(
        "<a href=\"/proxy?url={}\">{}</a>",
        urlencoding::encode(&resolved_href), display_text
    )
}

// スクリプトタグをスキップする関数（Rigil-Browserと同じ）
fn skip_script_tag(contents: &[char], i: &mut usize) {
    let mut tag = String::new();
    while *i < contents.len() {
        tag.push(contents[*i]);
        if contents[*i] == '>' && tag.contains("</script>") {
            break;
        }
        *i += 1;
    }
}

// スタイルタグをスキップする関数（Rigil-Browserと同じ）
fn skip_style_tag(contents: &[char], i: &mut usize) {
    let mut tag = String::new();
    while *i < contents.len() {
        tag.push(contents[*i]);
        if contents[*i] == '>' && tag.contains("</style>") {
            break;
        }
        *i += 1;
    }
}

// HTMLを解析してテキストに変換する関数（Rigil-Browserと同じ）
fn parse_html_to_text(html: &str, base_url: &str, current_url: &str) -> String {
    let contents: Vec<char> = html.chars().collect();
    let mut formatted_text = String::new();
    let mut i = 0;

    // 基本的なHTMLヘッダーを追加
    formatted_text.push_str("<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><style>body{font-family:'Segoe UI',Tahoma,Geneva,Verdana,sans-serif;line-height:1.6;margin:20px;color:#333;background-color:#fafafa;} a{color:#666;text-decoration:underline;margin-right:8px;} a:hover{color:#333;}</style></head><body>");

    while i < contents.len() {
        if contents[i] == '<' {
            let mut tag = String::new();

            // タグを読み取り
            while i < contents.len() {
                tag.push(contents[i]);
                i += 1;
                if contents[i-1] == '>' {
                    break;
                }
            }

            // タグの種類に応じて処理
            let tag_lower = tag.to_lowercase();
            if tag_lower.contains("<a ") || tag_lower == "<a>" {
                let link_html = process_link_tag(&tag, &contents, &mut i, base_url, current_url);
                if !link_html.is_empty() {
                    formatted_text.push_str(&link_html);
                }
            } else if tag_lower.contains("<script") {
                skip_script_tag(&contents, &mut i);
            } else if tag_lower.contains("<style") {
                skip_style_tag(&contents, &mut i);
            } else {
                formatted_text.push_str(&is_formatted(tag));
            }
        } else {
            // 通常のテキスト
            formatted_text.push(contents[i]);
            i += 1;
        }
    }

    formatted_text.push_str("</body></html>");
    formatted_text
}

// HTMLを取得する関数（非同期版）
async fn get_html(url: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    // URLからクエリパラメータを分離
    let parsed_url = match Url::parse(url) {
        Ok(u) => u,
        Err(e) => return Err(format!("URL解析エラー: {}", e)),
    };

    let base_url = format!("{}://{}{}", parsed_url.scheme(), parsed_url.host_str().unwrap_or(""), parsed_url.path());
    let query_pairs: Vec<(String, String)> = parsed_url.query_pairs().into_owned().collect();

    match client.get(&base_url).query(&query_pairs).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("レスポンス読み取りエラー: {}", e)),
            }
        }
        Err(e) => Err(format!("リクエストエラー: {}", e)),
    }
}

// フォーマットされたタグかどうかを判定する関数（Rigil-Browserと同じ）
fn is_formatted(tag: String) -> String {
    let tags: Vec<&str> = vec![
        "<title", "</title", "<br", "<br /", "<h1", "</h1", "<h2", "</h2", "<h3", "</h3", "<h4",
        "</h4", "<h5", "</h5", "<h6", "</h6", "<b>", "</b>", "<i>", "</i>", "<li>", "<li ",
        "</li>", "<ul", "</ul", "<ol", "<ol ", "</ol",
    ];
    let length_tags: usize = tags.len();
    for i in 0..length_tags - 1 {
        if tag.contains(tags[i]) {
            let output: String = tags[i].to_string();
            let vec_output: Vec<char> = output.chars().collect();
            let length_output: usize = output.len();
            if vec_output[length_output - 1] == '>' {
                return output;
            } else {
                return format!("{}{}", output, ">");
            }
        }
    }
    String::from("")
}
