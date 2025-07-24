// ========== ホームページ ==========
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
            margin: 40px auto;
            max-width: 600px;
            background-color: #fafafa;
            color: #333;
            line-height: 1.6;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 4px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        input[type="text"] {
            width: 70%;
            padding: 10px;
            font-size: 14px;
            border: 1px solid #ccc;
            border-radius: 4px;
            margin-right: 10px;
        }
        button {
            padding: 10px 20px;
            font-size: 14px;
            background-color: #007bff;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        button:hover {
            background-color: #0056b3;
        }
        .api-key-input {
            margin: 20px 0;
            padding: 15px;
            background-color: #f8f9fa;
            border-radius: 4px;
            border: 1px solid #dee2e6;
        }
        .url-input {
            margin: 20px 0;
        }
        .result {
            margin-top: 15px;
            padding: 10px;
            border-radius: 4px;
            display: none;
        }
        .error { background-color: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; }
        .success { background-color: #d4edda; color: #155724; border: 1px solid #c3e6cb; }
        h1 {
            color: #333;
            border-bottom: 2px solid #007bff;
            padding-bottom: 10px;
        }
        .api-info {
            background: #e7f3ff;
            padding: 15px;
            margin: 20px 0;
            border-radius: 4px;
            border-left: 4px solid #007bff;
        }
        .admin-link {
            text-align: center;
            margin: 20px 0;
        }
        .admin-link a {
            background-color: #6c757d;
            color: white;
            padding: 10px 20px;
            text-decoration: none;
            border-radius: 4px;
            display: inline-block;
        }
        .admin-link a:hover {
            background-color: #545b62;
        }
        .info-box {
            background: #fff3cd;
            color: #856404;
            padding: 15px;
            border-radius: 4px;
            border: 1px solid #ffeaa7;
            margin: 20px 0;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Rigil Proxy - HTML軽量化</h1>
        
        <div class="api-key-input">
            <label for="apiKey"><strong>APIキー:</strong></label><br>
            <input type="text" id="apiKey" placeholder="APIキーを入力" style="width: 80%; margin-top: 5px;">
            <button onclick="saveApiKey()">保存</button>
        </div>

        <div class="url-input">
            <label for="url"><strong>URL:</strong></label><br>
            <input type="text" id="url" placeholder="https://example.com" style="width: 80%; margin-top: 5px;">
            <button onclick="processUrl()">軽量化</button>
        </div>

        <div id="result" class="result"></div>

        <div class="admin-link">
            <a href="/admin">🔧 管理画面</a>
        </div>

        <div class="api-info">
            <h3>API使用方法</h3>
            <p><strong>GET:</strong> <code>/proxy?url=https://example.com&api_key=your_key</code></p>
            <p><strong>JSON API:</strong> <code>/api/process?url=https://example.com&api_key=your_key</code></p>
        </div>
    </div>

    <script>
        // ========== 設定 ==========
        const STORAGE_KEY = 'rigil_api_key';
        
        // ========== 状態管理 ==========
        let savedApiKey = localStorage.getItem(STORAGE_KEY) || '';
        
        // ========== 初期化 ==========
        window.onload = function() {
            if (savedApiKey) {
                document.getElementById('apiKey').value = savedApiKey;
            }
        };

        // ========== UI制御関数 ==========
        function showResult(message, type) {
            const result = document.getElementById('result');
            result.textContent = message;
            result.className = `result ${type}`;
            result.style.display = 'block';
        }

        function getInputValue(id) {
            return document.getElementById(id).value.trim();
        }

        // ========== APIキー管理 ==========
        function saveApiKey() {
            const apiKey = getInputValue('apiKey');
            if (!apiKey) {
                showResult('APIキーを入力してください', 'error');
                return;
            }
            localStorage.setItem(STORAGE_KEY, apiKey);
            savedApiKey = apiKey;
            showResult('APIキーを保存しました', 'success');
        }

        // ========== URL処理 ==========
        async function processUrl() {
            const url = getInputValue('url');
            const apiKey = getInputValue('apiKey') || savedApiKey;

            if (!url) {
                showResult('URLを入力してください', 'error');
                return;
            }
            
            if (!apiKey) {
                showResult('APIキーを入力してください', 'error');
                return;
            }

            showResult('処理中...', 'success');

            try {
                const response = await fetch(`/proxy?url=${encodeURIComponent(url)}&api_key=${encodeURIComponent(apiKey)}`);
                
                if (response.ok) {
                    const html = await response.text();
                    const newWindow = window.open();
                    if (newWindow) {
                        newWindow.document.write(html);
                        newWindow.document.close();
                        showResult('軽量化完了！新しいウィンドウで表示しました', 'success');
                    } else {
                        showResult('軽量化完了！ポップアップがブロックされました', 'success');
                    }
                } else {
                    const errorText = await response.text();
                    showResult(`エラー: ${response.status} - ${errorText}`, 'error');
                }
            } catch (error) {
                showResult(`エラー: ${error.message}`, 'error');
            }
        }

        // ========== キーボードイベント ==========
        document.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                if (document.activeElement.id === 'apiKey') {
                    saveApiKey();
                } else if (document.activeElement.id === 'url') {
                    processUrl();
                }
            }
        });
    </script>
</body>
</html>
    "#
}

// ========== APIドキュメント ==========
pub fn get_api_docs_html() -> &'static str {
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rigil Proxy API</title>
    <meta charset="UTF-8">
    <style>
        body { font-family: 'Segoe UI', sans-serif; margin: 40px auto; max-width: 800px; line-height: 1.6; }
        code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
        pre { background: #f4f4f4; padding: 15px; border-radius: 4px; overflow-x: auto; }
        h1 { border-bottom: 2px solid #007bff; padding-bottom: 10px; }
    </style>
</head>
<body>
    <h1>Rigil Proxy API</h1>
    
    <h2>エンドポイント</h2>
    
    <h3>HTML軽量化</h3>
    <p><strong>GET</strong> <code>/proxy?url=https://example.com&api_key=your_key</code></p>
    <p>軽量化されたHTMLを返します。</p>

    <h3>JSON API</h3>
    <p><strong>GET</strong> <code>/api/process?url=https://example.com&api_key=your_key</code></p>
    <p>JSON形式で結果を返します：</p>
    <pre>{
  "success": true,
  "data": "&lt;html&gt;...&lt;/html&gt;",
  "error": null,
  "original_url": "https://example.com",
  "processed_at": "2024-01-01T12:00:00Z"
}</pre>

    <p><a href="/">← ホームに戻る</a></p>
</body>
</html>
    "#
}

// ========== 管理画面 ==========
pub fn get_admin_page_html() -> &'static str {
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rigil Proxy - 管理画面</title>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 40px auto;
            max-width: 1000px;
            background-color: #fafafa;
            color: #333;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 4px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        .login-section {
            text-align: center;
            padding: 40px;
            background-color: #f8f9fa;
            border-radius: 8px;
            border: 1px solid #e9ecef;
            max-width: 400px;
            margin: 0 auto;
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
            margin: 10px;
        }
        button {
            padding: 12px 24px;
            font-size: 14px;
            background-color: #007bff;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            margin: 5px;
        }
        button:hover {
            background-color: #0056b3;
        }
        .danger-btn {
            background-color: #dc3545;
        }
        .danger-btn:hover {
            background-color: #c82333;
        }
        .secondary-btn {
            background-color: #6c757d;
        }
        .secondary-btn:hover {
            background-color: #545b62;
        }
        .logout-btn {
            background-color: #ffc107;
            color: #212529;
        }
        .logout-btn:hover {
            background-color: #e0a800;
        }
        .api-key-table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
            font-size: 14px;
        }
        .api-key-table th, .api-key-table td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        .api-key-table th {
            background-color: #f8f9fa;
        }
        .api-key-table tr:hover {
            background-color: #f5f5f5;
        }
        .result {
            margin: 15px 0;
            padding: 15px;
            border-radius: 4px;
            display: none;
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
        h1 {
            color: #333;
            border-bottom: 2px solid #007bff;
            padding-bottom: 10px;
        }
        .form-group {
            margin: 20px 0;
        }
        .form-group label {
            display: block;
            margin-bottom: 5px;
            font-weight: 500;
        }
        .stats-container {
            display: flex;
            gap: 20px;
            margin: 20px 0;
            flex-wrap: wrap;
        }
        .stat-card {
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            border: 1px solid #e9ecef;
            min-width: 180px;
            text-align: center;
            flex: 1;
        }
        .stat-value {
            font-size: 28px;
            font-weight: bold;
            margin-bottom: 8px;
        }
        .stat-label {
            color: #666;
            font-size: 14px;
        }
        .compression-ratio {
            color: #28a745;
        }
        .total-bytes {
            color: #dc3545;
        }
        .processed-bytes {
            color: #007bff;
        }
        .total-keys {
            color: #6f42c1;
        }
        .total-compressions {
            color: #fd7e14;
        }
        .header-actions {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
        }
    </style>
</head>
<body>
    <div class="container">
        <!-- ログイン画面 -->
        <div id="loginSection" class="login-section">
            <h1>🔒 管理者ログイン</h1>
            <p>管理者キーを入力してください</p>
            <div>
                <input type="password" id="adminKeyInput" placeholder="管理者キー" onkeypress="if(event.key==='Enter') login()">
                <br>
                <button onclick="login()">ログイン</button>
            </div>
            <div id="loginResult" class="result"></div>
            
            <div style="margin-top: 30px;">
                <a href="/" style="color: #6c757d; text-decoration: none;">← ホームに戻る</a>
            </div>
        </div>

        <!-- 管理画面 -->
        <div id="adminSection" class="admin-section">
            <div class="header-actions">
                <h1>🔧 Rigil Proxy 管理画面</h1>
                <div>
                    <button onclick="loadStatistics()" class="secondary-btn">🔄 データ更新</button>
                    <button onclick="logout()" class="logout-btn">ログアウト</button>
                </div>
            </div>

            <h2>📊 圧縮統計サマリー</h2>
            <div id="statisticsContainer" class="stats-container">
                <div class="stat-card">
                    <div class="stat-value total-keys" id="totalKeys">-</div>
                    <div class="stat-label">総APIキー数</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value total-bytes" id="totalOriginalSize">-</div>
                    <div class="stat-label">総原データ容量</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value processed-bytes" id="totalProcessedSize">-</div>
                    <div class="stat-label">総圧縮後容量</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value compression-ratio" id="compressionRatio">-</div>
                    <div class="stat-label">圧縮効率</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value total-compressions" id="totalCompressions">-</div>
                    <div class="stat-label">総圧縮回数</div>
                </div>
            </div>

            <h2>📋 APIキー一覧</h2>
            <button onclick="loadApiKeys()">🔄 更新</button>
            <div id="apiKeysResult" class="result"></div>
            <div id="apiKeysContainer">
                <p>読み込み中...</p>
            </div>

            <h2>➕ 新しいAPIキーを追加</h2>
            <div class="form-group">
                <label for="newApiKey">APIキー名:</label>
                <input type="text" id="newApiKey" placeholder="例: user-123-key">
                <button onclick="generateRandomKey()">ランダム生成</button>
                <button onclick="createApiKey()">追加</button>
            </div>
            <div id="createResult" class="result"></div>

            <div style="text-align: center; margin-top: 30px;">
                <a href="/" style="color: #6c757d; text-decoration: none;">← ホームに戻る</a>
            </div>
        </div>
    </div>

    <script>
        // ========== 設定 ==========
        const ADMIN_SESSION_KEY = 'rigil_admin_key';
        const API_ENDPOINTS = {
            login: '/api/admin/login',
            statistics: '/api/statistics',
            keysList: '/api/keys/list',
            keysCreate: '/api/keys/create',
            keysDelete: '/api/keys/delete'
        };

        // ========== 状態管理 ==========
        let currentAdminKey = '';

        // ========== 初期化 ==========
        window.onload = function() {
            const savedAdminKey = sessionStorage.getItem(ADMIN_SESSION_KEY);
            if (savedAdminKey) {
                currentAdminKey = savedAdminKey;
                showAdminSection();
            } else {
                showLoginSection();
            }
        };

        // ========== UI制御 ==========
        function showLoginSection() {
            document.getElementById('loginSection').style.display = 'block';
            document.getElementById('adminSection').style.display = 'none';
        }

        function showAdminSection() {
            document.getElementById('loginSection').style.display = 'none';
            document.getElementById('adminSection').style.display = 'block';
            loadApiKeys();
            loadStatistics();
        }

        function showResult(element, message, type) {
            element.textContent = message;
            element.className = `result ${type}`;
            element.style.display = 'block';
        }

        function hideResult(element) {
            element.style.display = 'none';
        }

        // ========== 認証機能 ==========
        async function login() {
            const adminKey = document.getElementById('adminKeyInput').value.trim();
            const resultBox = document.getElementById('loginResult');

            if (!adminKey) {
                showResult(resultBox, '管理者キーを入力してください', 'error');
                return;
            }

            try {
                const response = await apiRequest(API_ENDPOINTS.login, 'POST', {
                    admin_key: adminKey
                });

                if (response.success) {
                    currentAdminKey = adminKey;
                    sessionStorage.setItem(ADMIN_SESSION_KEY, adminKey);
                    showResult(resultBox, 'ログイン成功！', 'success');
                    
                    setTimeout(() => {
                        showAdminSection();
                    }, 1000);
                } else {
                    showResult(resultBox, `ログイン失敗: ${response.error}`, 'error');
                }
            } catch (error) {
                showResult(resultBox, `ネットワークエラー: ${error.message}`, 'error');
            }
        }

        function logout() {
            currentAdminKey = '';
            sessionStorage.removeItem(ADMIN_SESSION_KEY);
            document.getElementById('adminKeyInput').value = '';
            showLoginSection();
        }

        // ========== API通信 ==========
        async function apiRequest(url, method = 'GET', body = null) {
            const options = {
                method,
                headers: {
                    'Content-Type': 'application/json',
                }
            };

            if (body) {
                options.body = JSON.stringify(body);
            }

            const response = await fetch(url, options);
            return await response.json();
        }

        async function adminApiRequest(url) {
            if (!currentAdminKey) {
                logout();
                return null;
            }

            const urlWithAuth = `${url}${url.includes('?') ? '&' : '?'}admin_key=${encodeURIComponent(currentAdminKey)}`;
            const response = await fetch(urlWithAuth);
            const data = await response.json();

            if (data.error && data.error.includes('管理者権限')) {
                logout();
                return null;
            }

            return data;
        }

        // ========== 統計機能 ==========
        async function loadStatistics() {
            try {
                const data = await adminApiRequest(API_ENDPOINTS.statistics);
                if (data && data.success && data.statistics) {
                    updateStatisticsDisplay(data.statistics);
                }
            } catch (error) {
                console.error('統計データの読み込みエラー:', error);
            }
        }

        function updateStatisticsDisplay(stats) {
            document.getElementById('totalKeys').textContent = stats.total_keys.toLocaleString();
            document.getElementById('totalOriginalSize').textContent = formatBytes(stats.total_original_bytes);
            document.getElementById('totalProcessedSize').textContent = formatBytes(stats.total_processed_bytes);
            document.getElementById('compressionRatio').textContent = stats.compression_ratio + '%';
            document.getElementById('totalCompressions').textContent = stats.total_compressions.toLocaleString();
        }

        // ========== APIキー管理 ==========
        async function loadApiKeys() {
            const container = document.getElementById('apiKeysContainer');
            const resultBox = document.getElementById('apiKeysResult');

            try {
                const data = await adminApiRequest(API_ENDPOINTS.keysList);
                if (!data) return;

                if (data.success && data.keys) {
                    if (data.keys.length === 0) {
                        container.innerHTML = '<p style="color: #666;">APIキーが登録されていません</p>';
                    } else {
                        container.innerHTML = generateApiKeysTable(data.keys);
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

        function generateApiKeysTable(keys) {
            const rows = keys.map(key => {
                const originalBytes = key.total_original_bytes;
                const processedBytes = key.total_processed_bytes;
                const compressionRatio = originalBytes > 0 ? 
                    ((originalBytes - processedBytes) / originalBytes * 100) : 0;
                
                return `
                    <tr>
                        <td><code>${key.key}</code></td>
                        <td style="font-family: monospace;">${formatBytes(originalBytes)}</td>
                        <td style="font-family: monospace;">${formatBytes(processedBytes)}</td>
                        <td style="color: ${getCompressionColor(compressionRatio)}; font-weight: bold;">
                            ${compressionRatio.toFixed(1)}%
                        </td>
                        <td>${key.compression_count.toLocaleString()} 回</td>
                        <td>${new Date(key.created_at).toLocaleString('ja-JP')}</td>
                        <td>${key.last_used ? new Date(key.last_used).toLocaleString('ja-JP') : '未使用'}</td>
                        <td>
                            <button onclick="deleteApiKey('${key.key}')" class="danger-btn">削除</button>
                        </td>
                    </tr>
                `;
            }).join('');

            return `
                <table class="api-key-table">
                    <thead>
                        <tr>
                            <th>APIキー</th>
                            <th>使用量 (原データ)</th>
                            <th>圧縮後容量</th>
                            <th>圧縮効率</th>
                            <th>圧縮回数</th>
                            <th>作成日</th>
                            <th>最終使用</th>
                            <th>操作</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${rows}
                    </tbody>
                </table>
            `;
        }

        function generateRandomKey() {
            const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
            let result = 'key-';
            for (let i = 0; i < 16; i++) {
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

            try {
                const data = await apiRequest(API_ENDPOINTS.keysCreate, 'POST', {
                    key: apiKey
                });

                if (data.success) {
                    showResult(resultBox, `APIキー "${apiKey}" を追加しました！`, 'success');
                    newKeyInput.value = '';
                    loadApiKeys();
                    loadStatistics();
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

            const resultBox = document.getElementById('apiKeysResult');

            try {
                const url = `${API_ENDPOINTS.keysDelete}?key=${encodeURIComponent(apiKey)}&admin_key=${encodeURIComponent(currentAdminKey)}`;
                const response = await fetch(url, { method: 'DELETE' });
                const data = await response.json();

                if (data.success) {
                    showResult(resultBox, `APIキー "${apiKey}" を削除しました`, 'success');
                    loadApiKeys();
                    loadStatistics();
                } else if (data.error && data.error.includes('管理者権限')) {
                    logout();
                } else {
                    showResult(resultBox, `削除エラー: ${data.error}`, 'error');
                }
            } catch (error) {
                showResult(resultBox, `削除エラー: ${error.message}`, 'error');
            }
        }

        // ========== ユーティリティ関数 ==========
        function formatBytes(bytes) {
            if (bytes === 0) return '0 bytes';
            const k = 1024;
            const sizes = ['bytes', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
        }

        function getCompressionColor(ratio) {
            if (ratio > 50) return '#28a745';
            if (ratio > 20) return '#fd7e14';
            return '#dc3545';
        }
    </script>
</body>
</html>
"#
}
