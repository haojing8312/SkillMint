/// Tavily Search API Provider（占位实现）
///
/// 文档：https://docs.tavily.com/
/// 正式实现将在 Task 3-7 中完成。
use anyhow::Result;

use super::{SearchParams, SearchProvider, SearchResponse};

/// Tavily Search Provider
pub struct TavilySearch {
    /// API 基础 URL，默认 https://api.tavily.com
    pub base_url: String,
    /// Tavily API 密钥
    pub api_key: String,
}

impl TavilySearch {
    /// 创建 TavilySearch 实例
    ///
    /// - `base_url`：为空时使用默认地址
    /// - `api_key`：Tavily API 密钥
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let url = if base_url.is_empty() {
            "https://api.tavily.com".to_string()
        } else {
            base_url.to_string()
        };
        Self {
            base_url: url,
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for TavilySearch {
    fn name(&self) -> &str {
        "tavily"
    }

    fn display_name(&self) -> &str {
        "Tavily Search"
    }

    fn search(&self, _params: &SearchParams) -> Result<SearchResponse> {
        anyhow::bail!("Tavily Search Provider 尚未实现，将在 Task 3-7 中完成")
    }
}
