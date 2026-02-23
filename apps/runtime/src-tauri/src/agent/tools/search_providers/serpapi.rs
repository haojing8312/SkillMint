/// SerpApi 搜索 Provider（占位实现）
///
/// 文档：https://serpapi.com/
/// 支持多种搜索引擎（google、bing、baidu 等），通过 engine 参数指定。
/// 正式实现将在 Task 3-7 中完成。
use anyhow::Result;

use super::{SearchParams, SearchProvider, SearchResponse};

/// SerpApi 搜索 Provider
pub struct SerpApiSearch {
    /// API 基础 URL，默认 https://serpapi.com
    pub base_url: String,
    /// SerpApi API 密钥
    pub api_key: String,
    /// 搜索引擎类型（如 "google"、"bing"、"baidu"），空字符串时默认为 "google"
    pub engine: String,
}

impl SerpApiSearch {
    /// 创建 SerpApiSearch 实例
    ///
    /// - `base_url`：为空时使用默认地址
    /// - `api_key`：SerpApi API 密钥
    /// - `engine`：搜索引擎类型，为空时默认使用 "google"
    pub fn new(base_url: &str, api_key: &str, engine: &str) -> Self {
        let url = if base_url.is_empty() {
            "https://serpapi.com".to_string()
        } else {
            base_url.to_string()
        };
        let eng = if engine.is_empty() {
            "google".to_string()
        } else {
            engine.to_string()
        };
        Self {
            base_url: url,
            api_key: api_key.to_string(),
            engine: eng,
        }
    }
}

impl SearchProvider for SerpApiSearch {
    fn name(&self) -> &str {
        "serpapi"
    }

    fn display_name(&self) -> &str {
        "SerpApi"
    }

    fn search(&self, _params: &SearchParams) -> Result<SearchResponse> {
        anyhow::bail!("SerpApi Search Provider 尚未实现，将在 Task 3-7 中完成")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_engine() {
        // engine 为空时应默认为 "google"
        let provider = SerpApiSearch::new("", "key", "");
        assert_eq!(provider.engine, "google");
    }

    #[test]
    fn test_custom_engine() {
        // 指定 engine 时应保留原值
        let provider = SerpApiSearch::new("", "key", "bing");
        assert_eq!(provider.engine, "bing");
    }
}
