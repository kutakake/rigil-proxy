use url::Url;

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
pub fn parse_html_to_text(html: &str, base_url: &str, current_url: &str) -> String {
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
pub async fn get_html(url: &str) -> Result<String, String> {
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
        "</li>", "<ul", "</ul", "<ol", "<ol ", "</ol", "<code", "</code", "<pre", "</pre",
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
