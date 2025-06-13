pub fn get_home_page_html() -> &'static str {
    r#"
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
            <h2>管理機能</h2>
            <p>APIキーの管理は管理者画面で行えます：</p>
            <div style="text-align: center; margin: 20px 0;">
                <a href="/admin" style="background-color: #007bff; color: white; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block;">🔒 管理者画面へ</a>
            </div>
            <p style="font-size: 14px; color: #666;">※ 管理者キーが必要です</p>
        </div>



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
            "#
}

pub fn get_api_docs_html() -> &'static str {
    r#"
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
            "#
}

pub fn get_admin_page_html() -> &'static str {
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rigil Proxy - 管理者画面</title>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 40px;
            background-color: #fafafa;
            color: #333;
        }
        .container {
            max-width: 900px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 4px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        .login-section {
            text-align: center;
            padding: 40px;
            background-color: #f8f9fa;
            border-radius: 4px;
            border: 1px solid #e9ecef;
        }
        .admin-section {
            display: none;
        }
        input[type="text"], input[type="password"] {
            width: 300px;
            padding: 12px;
            font-size: 14px;
            border: 1px solid #ccc;
            border-radius: 4px;
            font-family: inherit;
            margin: 10px;
        }
        button {
            padding: 12px 24px;
            font-size: 14px;
            margin: 10px;
            background-color: #007bff;
            color: white;
            border: 1px solid #007bff;
            border-radius: 4px;
            cursor: pointer;
            font-family: inherit;
        }
        button:hover {
            background-color: #0056b3;
            border-color: #004085;
        }
        .danger-btn {
            background-color: #dc3545;
            border-color: #dc3545;
        }
        .danger-btn:hover {
            background-color: #c82333;
            border-color: #bd2130;
        }
        .secondary-btn {
            background-color: #6c757d;
            border-color: #6c757d;
        }
        .secondary-btn:hover {
            background-color: #545b62;
            border-color: #4e555b;
        }
        .api-key-table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }
        .api-key-table th, .api-key-table td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        .api-key-table th {
            background-color: #f8f9fa;
            font-weight: normal;
        }
        .api-key-table tr:hover {
            background-color: #f5f5f5;
        }
        .result-box {
            margin: 15px 0;
            padding: 15px;
            border-radius: 4px;
            min-height: 20px;
        }
        .success {
            color: #155724;
            background-color: #d4edda;
            border: 1px solid #c3e6cb;
        }
        .error {
            color: #721c24;
            background-color: #f8d7da;
            border: 1px solid #f5c6cb;
        }
        .info {
            color: #0c5460;
            background-color: #d1ecf1;
            border: 1px solid #b8daff;
        }
        h1 {
            color: #333;
            border-bottom: 2px solid #007bff;
            padding-bottom: 10px;
            font-weight: normal;
        }
        h2 {
            color: #555;
            font-weight: normal;
            font-size: 18px;
            margin-top: 30px;
        }
        .form-group {
            margin: 20px 0;
        }
        .form-group label {
            display: block;
            margin-bottom: 5px;
            font-weight: 500;
        }
        .actions {
            display: flex;
            gap: 10px;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔒 Rigil Proxy 管理者画面</h1>
        
        <!-- ログイン画面 -->
        <div id="loginSection" class="login-section">
            <h2>管理者認証</h2>
            <p>管理者キーを入力してください</p>
            <div>
                <input type="password" id="adminKeyInput" placeholder="管理者キー" onkeypress="if(event.key==='Enter') login()">
                <br>
                <button onclick="login()">ログイン</button>
            </div>
            <div id="loginResult" class="result-box" style="display: none;"></div>
        </div>

        <!-- 管理画面 -->
        <div id="adminSection" class="admin-section">
            <div class="actions">
                <button onclick="loadApiKeys()" class="secondary-btn">🔄 更新</button>
                <button onclick="logout()" class="danger-btn">ログアウト</button>
            </div>

            <h2>📋 APIキー一覧</h2>
            <div id="apiKeysResult" class="result-box" style="display: none;"></div>
            <div id="apiKeysContainer">
                <p>読み込み中...</p>
            </div>

            <h2>➕ 新しいAPIキーを作成</h2>
            <div class="form-group">
                <label for="newApiKey">APIキー名:</label>
                <input type="text" id="newApiKey" placeholder="例: user-123-key">
                <button onclick="generateRandomKey()">ランダム生成</button>
                <button onclick="createApiKey()">作成</button>
            </div>
            <div id="createResult" class="result-box" style="display: none;"></div>
        </div>
    </div>

    <script>
        let currentAdminKey = '';

        function login() {
            const adminKey = document.getElementById('adminKeyInput').value.trim();
            const resultBox = document.getElementById('loginResult');
            
            if (!adminKey) {
                showResult(resultBox, '管理者キーを入力してください', 'error');
                return;
            }
            
            currentAdminKey = adminKey;
            
            // 管理者キーを使ってAPIキー一覧を取得してみる（認証テスト）
            testAdminAccess();
        }

        async function testAdminAccess() {
            const resultBox = document.getElementById('loginResult');
            
            try {
                const response = await fetch(`/api/keys/list?admin_key=${encodeURIComponent(currentAdminKey)}`);
                const data = await response.json();
                
                if (data.success) {
                    // 認証成功
                    document.getElementById('loginSection').style.display = 'none';
                    document.getElementById('adminSection').style.display = 'block';
                    loadApiKeys();
                } else {
                    showResult(resultBox, `認証失敗: ${data.error}`, 'error');
                    currentAdminKey = '';
                }
            } catch (error) {
                showResult(resultBox, `ネットワークエラー: ${error.message}`, 'error');
                currentAdminKey = '';
            }
        }

        function logout() {
            currentAdminKey = '';
            document.getElementById('loginSection').style.display = 'block';
            document.getElementById('adminSection').style.display = 'none';
            document.getElementById('adminKeyInput').value = '';
            document.getElementById('loginResult').style.display = 'none';
        }

        async function loadApiKeys() {
            const container = document.getElementById('apiKeysContainer');
            const resultBox = document.getElementById('apiKeysResult');
            
            if (!currentAdminKey) {
                logout();
                return;
            }
            
            try {
                const response = await fetch(`/api/keys/list?admin_key=${encodeURIComponent(currentAdminKey)}`);
                const data = await response.json();
                
                if (data.success && data.keys) {
                    if (data.keys.length === 0) {
                        container.innerHTML = '<p class="info">APIキーが登録されていません</p>';
                    } else {
                        container.innerHTML = `
                            <table class="api-key-table">
                                <thead>
                                    <tr>
                                        <th>APIキー</th>
                                        <th>使用量 (bytes)</th>
                                        <th>作成日</th>
                                        <th>最終使用</th>
                                        <th>操作</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    ${data.keys.map(key => `
                                        <tr>
                                            <td><code>${key.key}</code></td>
                                            <td>${key.total_bytes_processed.toLocaleString()}</td>
                                            <td>${new Date(key.created_at).toLocaleString('ja-JP')}</td>
                                            <td>${key.last_used ? new Date(key.last_used).toLocaleString('ja-JP') : '未使用'}</td>
                                            <td>
                                                <button onclick="deleteApiKey('${key.key}')" class="danger-btn" style="padding: 6px 12px; margin: 0;">削除</button>
                                            </td>
                                        </tr>
                                    `).join('')}
                                </tbody>
                            </table>
                        `;
                    }
                    hideResult(resultBox);
                } else {
                    showResult(resultBox, `エラー: ${data.error}`, 'error');
                    container.innerHTML = '';
                }
            } catch (error) {
                showResult(resultBox, `ネットワークエラー: ${error.message}`, 'error');
                container.innerHTML = '';
            }
        }

        function generateRandomKey() {
            const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
            let result = 'key-';
            for (let i = 0; i < 32; i++) {
                result += chars.charAt(Math.floor(Math.random() * chars.length));
            }
            document.getElementById('newApiKey').value = result;
        }

        async function createApiKey() {
            const newKeyInput = document.getElementById('newApiKey');
            const resultBox = document.getElementById('createResult');
            const apiKey = newKeyInput.value.trim();
            
            if (!apiKey) {
                showResult(resultBox, 'APIキー名を入力してください', 'error');
                return;
            }
            
            if (!currentAdminKey) {
                logout();
                return;
            }
            
            try {
                const response = await fetch('/api/keys/create', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ 
                        admin_key: currentAdminKey,
                        key: apiKey 
                    })
                });
                
                const data = await response.json();
                
                if (data.success) {
                    showResult(resultBox, `APIキー "${apiKey}" を作成しました！`, 'success');
                    newKeyInput.value = '';
                    loadApiKeys(); // 一覧を更新
                } else {
                    showResult(resultBox, `エラー: ${data.error}`, 'error');
                }
            } catch (error) {
                showResult(resultBox, `ネットワークエラー: ${error.message}`, 'error');
            }
        }

        async function deleteApiKey(apiKey) {
            if (!confirm(`APIキー "${apiKey}" を削除してもよろしいですか？`)) {
                return;
            }
            
            if (!currentAdminKey) {
                logout();
                return;
            }
            
            const resultBox = document.getElementById('apiKeysResult');
            
            try {
                const response = await fetch(`/api/keys/delete?admin_key=${encodeURIComponent(currentAdminKey)}&key=${encodeURIComponent(apiKey)}`, {
                    method: 'DELETE'
                });
                
                const data = await response.json();
                
                if (data.success) {
                    showResult(resultBox, `APIキー "${apiKey}" を削除しました`, 'success');
                    loadApiKeys(); // 一覧を更新
                } else {
                    showResult(resultBox, `削除エラー: ${data.error}`, 'error');
                }
            } catch (error) {
                showResult(resultBox, `削除エラー: ${error.message}`, 'error');
            }
        }

        function showResult(element, message, type) {
            element.textContent = message;
            element.className = `result-box ${type}`;
            element.style.display = 'block';
        }

        function hideResult(element) {
            element.style.display = 'none';
        }

        // ページ読み込み時の処理
        document.addEventListener('DOMContentLoaded', function() {
            // 何もしない（ログイン画面から開始）
        });
    </script>
</body>
</html>
"#
} 