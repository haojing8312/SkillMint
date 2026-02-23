/// 秘塔搜索（Metaso）Provider
///
/// 文档：https://metaso.cn/
/// 支持两种响应格式，解析时自动适配。
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Instant;

use super::{SearchItem, SearchParams, SearchProvider, SearchResponse};

/// 秘塔搜索 Provider
pub struct MetasoSearch {
    /// API 基础 URL，默认 https://metaso.cn
    pub base_url: String,
    /// 秘塔搜索 API 密钥
    pub api_key: String,
}

impl MetasoSearch {
    /// 创建 MetasoSearch 实例
    ///
    /// - `base_url`：为空时使用默认地址，末尾 `/` 会自动去除
    /// - `api_key`：秘塔搜索 API 密钥
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let url = if base_url.is_empty() {
            "https://metaso.cn".to_string()
        } else {
            base_url.trim_end_matches('/').to_string()
        };
        Self {
            base_url: url,
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for MetasoSearch {
    fn name(&self) -> &str {
        "metaso"
    }

    fn display_name(&self) -> &str {
        "秘塔搜索"
    }

    fn search(&self, params: &SearchParams) -> Result<SearchResponse> {
        let start = Instant::now();

        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let body = json!({
            "query": params.query,
            "mode": "concise"
        });

        let url = format!("{}/api/search", self.base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        // 错误码映射
        let status = response.status();
        if !status.is_success() {
            let code = status.as_u16();
            return Err(anyhow!(
                "{}",
                match code {
                    401 | 403 => "搜索配置错误：API 密钥无效或权限不足".to_string(),
                    429 => "搜索频率超限：请稍后重试".to_string(),
                    500..=599 => "搜索服务暂不可用：服务器内部错误".to_string(),
                    _ => format!("搜索请求失败，HTTP 状态码: {}", code),
                }
            ));
        }

        let resp_body: Value = response.json()?;
        let items = parse_metaso_response(&resp_body);

        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "metaso".to_string(),
            items,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// 从秘塔搜索 API 响应中提取搜索结果
///
/// 支持两种响应格式：
/// - 格式 1：`{ data: { items: [{ title, url, content/snippet }] } }`
/// - 格式 2：`{ results: [{ title, url, snippet/content }] }`
fn parse_metaso_response(json: &Value) -> Vec<SearchItem> {
    // 优先尝试格式 1：data.items
    if let Some(items_arr) = json
        .get("data")
        .and_then(|d| d.get("items"))
        .and_then(|i| i.as_array())
    {
        return extract_items_from_array(items_arr);
    }

    // 回退到格式 2：results
    if let Some(results_arr) = json.get("results").and_then(|r| r.as_array()) {
        return extract_items_from_array(results_arr);
    }

    vec![]
}

/// 从搜索结果数组中提取 SearchItem 列表
///
/// 兼容 content/snippet 两种摘要字段名。
fn extract_items_from_array(arr: &[Value]) -> Vec<SearchItem> {
    arr.iter()
        .filter_map(|item| {
            let title = item.get("title")?.as_str()?.to_string();
            let url = item.get("url")?.as_str()?.to_string();
            // 优先使用 content，其次 snippet
            let snippet = item
                .get("content")
                .or_else(|| item.get("snippet"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            Some(SearchItem { title, url, snippet })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_format1_normal_response() {
        // 格式 1：data.items 数组，摘要字段为 content
        let json = json!({
            "data": {
                "items": [
                    {
                        "title": "秘塔搜索介绍",
                        "url": "https://metaso.cn/about",
                        "content": "秘塔搜索是一款 AI 搜索引擎"
                    },
                    {
                        "title": "秘塔 API 文档",
                        "url": "https://metaso.cn/api",
                        "snippet": "API 接口文档说明"
                    }
                ]
            }
        });

        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "秘塔搜索介绍");
        assert_eq!(items[0].url, "https://metaso.cn/about");
        assert_eq!(items[0].snippet, "秘塔搜索是一款 AI 搜索引擎");
        // 第二条使用 snippet 字段
        assert_eq!(items[1].snippet, "API 接口文档说明");
    }

    #[test]
    fn test_parse_format2_normal_response() {
        // 格式 2：顶层 results 数组，摘要字段为 snippet
        let json = json!({
            "results": [
                {
                    "title": "Rust 语言",
                    "url": "https://www.rust-lang.org",
                    "snippet": "Rust 是系统编程语言"
                },
                {
                    "title": "Cargo 文档",
                    "url": "https://doc.rust-lang.org/cargo",
                    "content": "Rust 包管理器"
                }
            ]
        });

        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Rust 语言");
        assert_eq!(items[0].url, "https://www.rust-lang.org");
        assert_eq!(items[0].snippet, "Rust 是系统编程语言");
        // 第二条使用 content 字段
        assert_eq!(items[1].snippet, "Rust 包管理器");
    }

    #[test]
    fn test_parse_format1_takes_priority_over_format2() {
        // 同时存在两种格式时，格式 1（data.items）优先
        let json = json!({
            "data": {
                "items": [
                    {
                        "title": "格式1结果",
                        "url": "https://format1.example.com",
                        "content": "来自格式1"
                    }
                ]
            },
            "results": [
                {
                    "title": "格式2结果",
                    "url": "https://format2.example.com",
                    "snippet": "来自格式2"
                }
            ]
        });

        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 1, "格式1优先，应只返回 data.items 中的结果");
        assert_eq!(items[0].title, "格式1结果");
    }

    #[test]
    fn test_parse_empty_format1() {
        // 格式 1 的 items 为空数组
        let json = json!({ "data": { "items": [] } });
        let items = parse_metaso_response(&json);
        // 空数组命中格式1，返回空列表
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_parse_empty_format2() {
        // 格式 2 的 results 为空数组
        let json = json!({ "results": [] });
        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 0, "空 results 应返回空列表");
    }

    #[test]
    fn test_parse_no_known_fields() {
        // 响应中既无 data.items 也无 results
        let json = json!({ "status": "ok", "query": "rust" });
        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 0, "未知格式应返回空列表");
    }

    #[test]
    fn test_parse_missing_snippet_and_content() {
        // title 和 url 存在，但 snippet 和 content 均缺失
        let json = json!({
            "results": [
                {
                    "title": "无摘要页面",
                    "url": "https://example.com"
                }
            ]
        });

        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].snippet, "");
    }

    #[test]
    fn test_new_trims_trailing_slash() {
        let provider = MetasoSearch::new("https://metaso.cn/", "key");
        assert_eq!(provider.base_url, "https://metaso.cn");
    }

    #[test]
    fn test_new_uses_default_url() {
        let provider = MetasoSearch::new("", "key");
        assert_eq!(provider.base_url, "https://metaso.cn");
    }
}
