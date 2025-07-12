use url::Url;
use reader_mode_maker;
use std::time::Duration;
use htmlescape;

// URLを正規化する関数（Rigil-Browserと同じ）
pub fn normalize_url(name: &str) -> String {
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
pub fn get_base_url(url: &str) -> String {
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
    let mut display_text = if link_content.trim().is_empty() {
        resolved_href.clone()
    } else {
        link_content.trim().to_string()
    };

    // 長いリンクテキストを短縮する（50文字以上の場合）
    if display_text.chars().count() > 50 {
        display_text = format!("{}...", display_text.chars().take(47).collect::<String>());
    }

    // プロキシ経由でリンクを処理するように修正
    format!(
        "<a href=\"/proxy?url={}\" title=\"{}\">{}</a>",
        urlencoding::encode(&resolved_href), 
        htmlescape::encode_minimal(&resolved_href),
        htmlescape::encode_minimal(&display_text)
    )
}

// HTMLを解析してテキストに変換する関数（Rigil-Browserと同じ）
pub fn parse_html_to_text(html: &str, base_url: &str, current_url: &str) -> String {
    let mut formatted_text = String::new();

    // 基本的なHTMLヘッダーを追加
    formatted_text.push_str("<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><style>body{font-family:'Segoe UI',Tahoma,Geneva,Verdana,sans-serif;line-height:1.6;margin:20px;color:#333;background-color:#fafafa;max-width:100%;overflow-x:auto;} a{color:#666;text-decoration:underline;margin-right:8px;word-break:break-word;max-width:100%;display:inline-block;} a:hover{color:#333;}</style></head><body>");

    let culled_html = reader_mode_maker::culling(html);
    let contents: Vec<char> = culled_html.chars().collect();
    let mut i = 0;

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
            } else {
                formatted_text.push_str(&tag);
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
pub async fn get_html(url: &str) -> Result<(String, String), String> {
    // タイムアウト設定を含むHTTPクライアントを作成
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))  // 30秒のタイムアウト
        .redirect(reqwest::redirect::Policy::limited(10))  // 最大10回のリダイレクト
        .build()
        .map_err(|e| format!("HTTPクライアントの作成エラー: {}", e))?;

    // URLからクエリパラメータを分離
    let parsed_url = match Url::parse(url) {
        Ok(u) => u,
        Err(e) => return Err(format!("URL解析エラー: {}", e)),
    };

    let base_url = format!("{}://{}{}", parsed_url.scheme(), parsed_url.host_str().unwrap_or(""), parsed_url.path());
    let query_pairs: Vec<(String, String)> = parsed_url.query_pairs().into_owned().collect();

    println!("HTMLを取得中: {}", url);
    
    match client.get(&base_url).query(&query_pairs).send().await {
        Ok(response) => {
            // ステータスコードをチェック
            if !response.status().is_success() {
                return Err(format!("HTTPエラー: {} - {}", response.status(), response.status().canonical_reason().unwrap_or("不明なエラー")));
            }

            // リダイレクト後の最終URLを取得
            let final_url = response.url().to_string();
            println!("最終URL: {}", final_url);
            
            match response.text().await {
                Ok(text) => {
                    println!("HTML取得完了: {} bytes", text.len());
                    Ok((text, final_url))
                },
                Err(e) => {
                    if e.is_timeout() {
                        Err("タイムアウトエラー: レスポンスの読み取りに時間がかかりすぎました".to_string())
                    } else {
                        Err(format!("レスポンス読み取りエラー: {}", e))
                    }
                }
            }
        }
        Err(e) => {
            if e.is_timeout() {
                Err("タイムアウトエラー: サーバーからの応答に時間がかかりすぎました".to_string())
            } else if e.is_connect() {
                Err("接続エラー: サーバーに接続できません".to_string())
            } else if e.is_request() {
                Err("リクエストエラー: 不正なリクエストです".to_string())
            } else {
                Err(format!("ネットワークエラー: {}", e))
            }
        }
    }
}
