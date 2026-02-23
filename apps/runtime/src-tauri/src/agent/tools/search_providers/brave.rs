/// Brave Search API Provider（占位实现）
///
/// 文档：https://api.search.brave.com/
/// 正式实现将在 Task 3-7 中完成。
use anyhow::Result;

use super::{SearchParams, SearchProvider, SearchResponse};

/// Brave Search Provider
pub struct BraveSearch {
    /// API 基础 URL，默认 https://api.search.brave.com
    pub base_url: String,
    /// Brave Search API 密钥
    pub api_key: String,
}

impl BraveSearch {
    /// 创建 BraveSearch 实例
    ///
    /// - `base_url`：为空时使用默认地址
    /// - `api_key`：Brave API 密钥
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let url = if base_url.is_empty() {
            "https://api.search.brave.com".to_string()
        } else {
            base_url.to_string()
        };
        Self {
            base_url: url,
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for BraveSearch {
    fn name(&self) -> &str {
        "brave"
    }

    fn display_name(&self) -> &str {
        "Brave Search"
    }

    fn search(&self, _params: &SearchParams) -> Result<SearchResponse> {
        anyhow::bail!("Brave Search Provider 尚未实现，将在 Task 3-7 中完成")
    }
}
