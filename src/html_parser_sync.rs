use reqwest::blocking::Client;
use std::time::Duration;

pub fn get_html_sync(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;
    
    let response = client.get(url).send()?;
    let html = response.text()?;
    Ok(html)
}

pub fn get_base_url(url: &str) -> String {
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(host) = parsed_url.host_str() {
            format!("{}://{}", parsed_url.scheme(), host)
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

pub fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

pub fn parse_html_to_text(html: &str, base_url: &str, _current_url: &str) -> String {
    // 簡単なHTMLパーサー（元のコードと同様の処理）
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut current_tag = String::new();
    
    let mut chars = html.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '<' {
            in_tag = true;
            current_tag.clear();
        } else if ch == '>' && in_tag {
            in_tag = false;
            
            let tag_lower = current_tag.to_lowercase();
            if tag_lower.starts_with("script") {
                in_script = true;
            } else if tag_lower == "/script" {
                in_script = false;
            } else if tag_lower.starts_with("style") {
                in_style = true;
            } else if tag_lower == "/style" {
                in_style = false;
            } else if tag_lower.starts_with("title") {
                result.push_str("\n【タイトル】");
            } else if tag_lower == "/title" {
                result.push_str("\n");
            } else if tag_lower.starts_with("h1") || tag_lower.starts_with("h2") || 
                      tag_lower.starts_with("h3") || tag_lower.starts_with("h4") || 
                      tag_lower.starts_with("h5") || tag_lower.starts_with("h6") {
                result.push_str("\n\n■ ");
            } else if tag_lower == "/h1" || tag_lower == "/h2" || tag_lower == "/h3" || 
                      tag_lower == "/h4" || tag_lower == "/h5" || tag_lower == "/h6" {
                result.push_str("\n");
            } else if tag_lower == "p" || tag_lower == "div" || tag_lower == "br" {
                result.push_str("\n");
            } else if tag_lower.starts_with("a ") {
                // リンクの処理
                if let Some(href_start) = tag_lower.find("href=\"") {
                    let href_start = href_start + 6;
                    if let Some(href_end) = tag_lower[href_start..].find("\"") {
                        let href = &current_tag[href_start..href_start + href_end];
                        let absolute_url = resolve_url(href, base_url);
                        result.push_str(&format!("[リンク: {}]", absolute_url));
                    }
                }
            }
            
            current_tag.clear();
        } else if in_tag {
            current_tag.push(ch);
        } else if !in_script && !in_style {
            if ch.is_whitespace() {
                if !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
            } else {
                result.push(ch);
            }
        }
    }
    
    // 連続する空白文字を整理
    let mut cleaned = String::new();
    let mut last_was_whitespace = false;
    
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !last_was_whitespace {
                cleaned.push(if ch == '\n' { '\n' } else { ' ' });
            }
            last_was_whitespace = true;
        } else {
            cleaned.push(ch);
            last_was_whitespace = false;
        }
    }
    
    cleaned.trim().to_string()
}

fn resolve_url(href: &str, base_url: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{}", href)
    } else if href.starts_with("/") {
        format!("{}{}", base_url, href)
    } else {
        format!("{}/{}", base_url, href)
    }
} 