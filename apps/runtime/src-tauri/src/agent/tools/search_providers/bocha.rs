/// 博查（Bocha）搜索 Provider（占位实现）
///
/// 文档：https://open.bochaai.com/
/// 正式实现将在 Task 3-7 中完成。
use anyhow::Result;

use super::{SearchParams, SearchProvider, SearchResponse};

/// 博查搜索 Provider
pub struct BochaSearch {
    /// API 基础 URL，默认 https://api.bochaai.com
    pub base_url: String,
    /// 博查搜索 API 密钥
    pub api_key: String,
}

impl BochaSearch {
    /// 创建 BochaSearch 实例
    ///
    /// - `base_url`：为空时使用默认地址
    /// - `api_key`：博查搜索 API 密钥
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let url = if base_url.is_empty() {
            "https://api.bochaai.com".to_string()
        } else {
            base_url.to_string()
        };
        Self {
            base_url: url,
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for BochaSearch {
    fn name(&self) -> &str {
        "bocha"
    }

    fn display_name(&self) -> &str {
        "博查搜索"
    }

    fn search(&self, _params: &SearchParams) -> Result<SearchResponse> {
        anyhow::bail!("博查搜索 Provider 尚未实现，将在 Task 3-7 中完成")
    }
}
