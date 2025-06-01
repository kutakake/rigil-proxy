# Rigil Proxy

Rigil-BrowserのHTML軽量化機能と同じ挙動を持つプロキシサーバーです。

## 機能

- HTMLの軽量化（不要なタグの除去）
- JavaScriptとCSSの除去
- リンクの変換（プロキシ経由でのナビゲーション）
- 相対URLの絶対URL変換
- 特定のHTMLタグのみを保持（title、br、h1-h6、b、i、ul、li、ol）
- **RESTful API対応**（JSON形式でのレスポンス）

## 使用方法

### サーバーの起動

```bash
cargo run
```

サーバーは `http://127.0.0.1:8080` で起動します。

### Webインターフェース

ブラウザで `http://127.0.0.1:8080` にアクセスすると、URLを入力するフォームが表示されます。

### API使用方法

#### 1. HTML軽量化 (GET)
```bash
curl "http://127.0.0.1:8080/proxy?url=https://example.com"
```
軽量化されたHTMLを直接返します。

#### 2. JSON API (GET)
```bash
curl "http://127.0.0.1:8080/api/process?url=https://example.com"
```
JSON形式で結果を返します：
```json
{
  "success": true,
  "data": "<html>...</html>",
  "error": null,
  "original_url": "https://example.com",
  "processed_at": "2024-01-01T12:00:00Z"
}
```

#### 3. JSON API (POST)
```bash
curl -X POST "http://127.0.0.1:8080/api/process" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

### APIドキュメント

詳細なAPIドキュメントは `http://127.0.0.1:8080/api/docs` で確認できます。

## 実装詳細

このプロキシは以下のRigil-Browserの機能を再現しています：

1. **URL正規化**: `http://`または`https://`が含まれていない場合、自動的に`https://`を追加
2. **ベースURL取得**: 相対URLの解決のためのベースURLを計算
3. **HTML解析**: HTMLを文字単位で解析し、タグを識別
4. **タグフィルタリング**: 許可されたタグのみを保持
5. **リンク変換**: `<a>`タグをプロキシ経由のリンクに変換
6. **スクリプト/スタイル除去**: `<script>`と`<style>`タグを完全に除去

## APIエンドポイント一覧

| エンドポイント | メソッド | 説明 | レスポンス形式 |
|---------------|---------|------|---------------|
| `/` | GET | Webインターフェース | HTML |
| `/api/docs` | GET | APIドキュメント | HTML |
| `/proxy` | GET | HTML軽量化 | HTML |
| `/api/process` | GET | JSON API (クエリパラメータ) | JSON |
| `/api/process` | POST | JSON API (リクエストボディ) | JSON |

## 依存関係

- `tokio`: 非同期ランタイム
- `hyper`: HTTPサーバー
- `reqwest`: HTTPクライアント
- `url`: URL解析
- `urlencoding`: URLエンコーディング
- `serde`: シリアライゼーション
- `chrono`: 日時処理

## 注意事項

- このプロキシはHTTPS証明書の検証を行います
- 一部のWebサイトはCORSポリシーにより正常に動作しない場合があります
- JavaScriptに依存するWebサイトは正常に表示されない場合があります（意図的な動作）

## 使用例

### curlでのAPI使用例

```bash
# HTML軽量化
curl "http://127.0.0.1:8080/proxy?url=https://news.ycombinator.com"

# JSON APIでの軽量化
curl "http://127.0.0.1:8080/api/process?url=https://news.ycombinator.com" | jq .

# POSTでのJSON API
curl -X POST "http://127.0.0.1:8080/api/process" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://news.ycombinator.com"}' | jq .
```

### JavaScriptでのAPI使用例

```javascript
// Fetch APIを使用
fetch('/api/process?url=https://example.com')
  .then(response => response.json())
  .then(data => {
    if (data.success) {
      console.log('軽量化されたHTML:', data.data);
    } else {
      console.error('エラー:', data.error);
    }
  });

// POST リクエスト
fetch('/api/process', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    url: 'https://example.com'
  })
})
.then(response => response.json())
.then(data => console.log(data));
``` 