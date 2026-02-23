/// 秘塔搜索（Metaso）Provider（占位实现）
///
/// 文档：https://metaso.cn/
/// 正式实现将在 Task 3-7 中完成。
use anyhow::Result;

use super::{SearchParams, SearchProvider, SearchResponse};

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
    /// - `base_url`：为空时使用默认地址
    /// - `api_key`：秘塔搜索 API 密钥
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let url = if base_url.is_empty() {
            "https://metaso.cn".to_string()
        } else {
            base_url.to_string()
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

    fn search(&self, _params: &SearchParams) -> Result<SearchResponse> {
        anyhow::bail!("秘塔搜索 Provider 尚未实现，将在 Task 3-7 中完成")
    }
}
