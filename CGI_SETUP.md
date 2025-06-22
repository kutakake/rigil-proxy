# Rigil Proxy CGI版 セットアップガイド

このガイドでは、Rigil ProxyのCGI版をWebサーバーにデプロイする方法を説明します。

## 1. ビルド

### 1.1 CGI版のビルド
```bash
# リリース版のビルド
cargo build --release --bin rigil-proxy-cgi

# ビルド成果物の確認
ls -la target/release/rigil-proxy-cgi
```

### 1.2 実行権限の付与
```bash
chmod +x target/release/rigil-proxy-cgi
```

## 2. Webサーバー設定

### 2.1 Apache設定例

#### .htaccessファイル
```apache
# .htaccess
Options +ExecCGI
AddHandler cgi-script .cgi

# URL書き換え設定
RewriteEngine On
RewriteCond %{REQUEST_FILENAME} !-f
RewriteCond %{REQUEST_FILENAME} !-d
RewriteRule ^(.*)$ rigil-proxy.cgi/$1 [L,QSA]
```

#### Apacheの設定ファイル (httpd.conf または site.conf)
```apache
<Directory "/var/www/html/rigil-proxy">
    Options +ExecCGI
    AllowOverride All
    Require all granted
    AddHandler cgi-script .cgi
</Directory>
```

### 2.2 Nginx + FCGIWrap設定例

#### nginx.conf
```nginx
location /rigil-proxy/ {
    gzip off;
    fastcgi_pass unix:/var/run/fcgiwrap.socket;
    include fastcgi_params;
    fastcgi_param SCRIPT_FILENAME /var/www/html/rigil-proxy/rigil-proxy.cgi;
    fastcgi_param PATH_INFO $fastcgi_path_info;
    fastcgi_param QUERY_STRING $query_string;
}
```

## 3. デプロイメント

### 3.1 ファイル配置
```bash
# Webサーバーのドキュメントルートにディレクトリを作成
sudo mkdir -p /var/www/html/rigil-proxy

# CGIバイナリをコピー
sudo cp target/release/rigil-proxy-cgi /var/www/html/rigil-proxy/rigil-proxy.cgi

# 設定ファイルをコピー
sudo cp api_keys.json /var/www/html/rigil-proxy/

# 実行権限を設定
sudo chmod +x /var/www/html/rigil-proxy/rigil-proxy.cgi
sudo chmod 666 /var/www/html/rigil-proxy/api_keys.json

# 所有者をWebサーバーユーザーに変更
sudo chown -R www-data:www-data /var/www/html/rigil-proxy
```

### 3.2 .htaccessファイルの作成（Apache使用時）
```bash
cat > /var/www/html/rigil-proxy/.htaccess << 'EOF'
Options +ExecCGI
AddHandler cgi-script .cgi

RewriteEngine On
RewriteCond %{REQUEST_FILENAME} !-f
RewriteCond %{REQUEST_FILENAME} !-d
RewriteRule ^(.*)$ rigil-proxy.cgi/$1 [L,QSA]
EOF
```

## 4. 使用方法

### 4.1 基本的な使用方法

#### WebUI
```
http://yourdomain.com/rigil-proxy/
```

#### プロキシ機能
```
http://yourdomain.com/rigil-proxy/proxy?url=https://example.com
```

#### API機能
```bash
# GET リクエスト
curl "http://yourdomain.com/rigil-proxy/api/process?url=https://example.com&api_key=YOUR_API_KEY"

# POST リクエスト
curl -X POST "http://yourdomain.com/rigil-proxy/api/process" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: YOUR_API_KEY" \
  -d '{"url":"https://example.com"}'
```

### 4.2 管理機能

#### APIキーの作成
```bash
curl -X POST "http://yourdomain.com/rigil-proxy/api/keys/create" \
  -H "X-Admin-Key: admin_rigil_proxy_master_key_2024"
```

#### 使用状況の確認
```bash
curl "http://yourdomain.com/rigil-proxy/api/keys/usage?api_key=YOUR_API_KEY"
```

## 5. トラブルシューティング

### 5.1 よくある問題

#### CGIが実行されない
- 実行権限が設定されているか確認
- Webサーバーの設定でCGIが有効になっているか確認
- SELinuxが有効な場合は、適切なコンテキストを設定

#### パーミッションエラー
```bash
# ファイルの所有者とパーミッションを確認
ls -la /var/www/html/rigil-proxy/

# Webサーバーユーザーでの実行テスト
sudo -u www-data /var/www/html/rigil-proxy/rigil-proxy.cgi
```

#### 500 Internal Server Errorが発生する場合
```bash
# Webサーバーのエラーログを確認
sudo tail -f /var/log/apache2/error.log
# または
sudo tail -f /var/log/nginx/error.log
```

### 5.2 デバッグ

#### CGIの手動実行テスト
```bash
# 環境変数を設定してテスト実行
export REQUEST_METHOD=GET
export QUERY_STRING="url=https://example.com"
export PATH_INFO="/"
cd /var/www/html/rigil-proxy
./rigil-proxy.cgi
```

## 6. セキュリティ考慮事項

### 6.1 ファイアウォール設定
- 必要なポート（80, 443）のみを開放
- 不要なサービスを停止

### 6.2 アクセス制御
- 管理者APIのアクセス制限
- APIキーの定期的な更新
- ログの監視

### 6.3 ファイルパーミッション
```bash
# セキュアなパーミッション設定
sudo chmod 755 /var/www/html/rigil-proxy/rigil-proxy.cgi
sudo chmod 644 /var/www/html/rigil-proxy/api_keys.json
sudo chown www-data:www-data /var/www/html/rigil-proxy/*
```

## 7. パフォーマンス最適化

### 7.1 キャッシュ設定
- 静的ファイルのキャッシュ設定
- CDNの利用検討

### 7.2 リソース制限
- メモリ使用量の制限
- 実行時間の制限設定

## 8. バックアップ

### 8.1 定期バックアップの設定
```bash
# crontabに追加
0 2 * * * cp /var/www/html/rigil-proxy/api_keys.json /backup/api_keys_$(date +\%Y\%m\%d).json
```

このガイドに従って設定することで、Rigil ProxyのCGI版を安全かつ効率的にデプロイできます。 