/// DuckDuckGo HTML 搜索 Provider（无需 API 密钥）
///
/// 通过解析 DuckDuckGo HTML 搜索结果页面实现，适合开发和无 API Key 的场景。
/// 生产环境建议使用付费 API Provider（Brave、Tavily 等）以获得更稳定的结果。
use anyhow::{anyhow, Result};
use std::time::Instant;

use super::{SearchItem, SearchParams, SearchProvider, SearchResponse};

/// DuckDuckGo HTML 搜索 Provider
pub struct DuckDuckGoSearch;

impl DuckDuckGoSearch {
    /// 创建 DuckDuckGo 搜索 Provider 实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for DuckDuckGoSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for DuckDuckGoSearch {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    fn display_name(&self) -> &str {
        "DuckDuckGo"
    }

    fn search(&self, params: &SearchParams) -> Result<SearchResponse> {
        let start = Instant::now();

        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(20))
            .build()?;

        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(&params.query)
        );

        let resp = client
            .get(&url)
            .header("User-Agent", "SkillMint/1.0")
            .send()
            .map_err(|e| anyhow!("搜索请求失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(anyhow!("搜索请求返回错误状态: {}", resp.status()));
        }

        let html = resp
            .text()
            .map_err(|e| anyhow!("读取搜索结果失败: {}", e))?;

        let items = parse_duckduckgo_html(&html, params.count);

        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "duckduckgo".to_string(),
            items,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// 解析 DuckDuckGo HTML 搜索结果页
fn parse_duckduckgo_html(html: &str, max_results: usize) -> Vec<SearchItem> {
    let mut results = Vec::new();

    // 匹配 result__a 链接（标题 + URL）
    let link_re =
        regex::Regex::new(r#"<a[^>]*class="result__a"[^>]*href="([^"]*)"[^>]*>([\s\S]*?)</a>"#)
            .unwrap();

    // 匹配 result__snippet（摘要）
    let snippet_re =
        regex::Regex::new(r#"<a[^>]*class="result__snippet"[^>]*>([\s\S]*?)</a>"#).unwrap();

    let html_tag_re = regex::Regex::new(r"<[^>]+>").unwrap();

    // 按 result__body 分割结果块
    let body_parts: Vec<&str> = html.split("result__body").collect();

    for part in body_parts.iter().skip(1) {
        if results.len() >= max_results {
            break;
        }

        let title;
        let raw_url;
        let snippet;

        if let Some(link_cap) = link_re.captures(part) {
            raw_url = link_cap[1].to_string();
            title = html_tag_re
                .replace_all(&link_cap[2], "")
                .trim()
                .to_string();
        } else {
            continue;
        }

        snippet = snippet_re
            .captures(part)
            .map(|cap| {
                html_tag_re
                    .replace_all(&cap[1], "")
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();

        // 解码 DuckDuckGo 重定向 URL
        let url = if raw_url.contains("uddg=") {
            raw_url
                .split("uddg=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
                .unwrap_or(raw_url)
        } else {
            raw_url
        };

        results.push(SearchItem {
            title,
            url,
            snippet,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        // 空 HTML 应返回空结果
        let items = parse_duckduckgo_html("", 5);
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_respects_max_results() {
        // max_results 为 0 时不返回任何结果
        let items = parse_duckduckgo_html("result__body some content result__body more", 0);
        assert_eq!(items.len(), 0);
    }
}
