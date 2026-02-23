# 多 Provider 网络搜索系统实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 web_search 工具从单一 DuckDuckGo HTML 解析升级为支持 5 个搜索 Provider（Brave、Tavily、秘塔、博查、SerpAPI）的生产级系统。

**Architecture:** Provider Trait 抽象，纯 Rust 实现。通过 `model_configs` 表（`api_format` 以 `search_` 前缀标识）存储搜索配置。SearchCache 提供内存缓存。chat.rs 在每次 `send_message` 时从 DB 动态加载搜索 Provider。

**Tech Stack:** Rust, reqwest (blocking), serde_json, regex, urlencoding, SQLite (sqlx)

**Design Doc:** `docs/plans/2026-02-23-web-search-providers-design.md`

---

## Task 1: SearchProvider Trait + 公共类型

**Files:**
- Create: `src/agent/tools/search_providers/mod.rs`
- Modify: `src/agent/tools/mod.rs` — 添加 `pub mod search_providers;`

**Step 1: 创建 search_providers 目录和 mod.rs**

```rust
// src/agent/tools/search_providers/mod.rs

pub mod brave;
pub mod tavily;
pub mod metaso;
pub mod bocha;
pub mod serpapi;

use anyhow::Result;

/// 搜索 Provider trait
pub trait SearchProvider: Send + Sync {
    /// Provider 标识符，如 "brave", "metaso"
    fn name(&self) -> &str;
    /// 显示名称，如 "Brave Search", "秘塔搜索"
    fn display_name(&self) -> &str;
    /// 执行搜索
    fn search(&self, params: &SearchParams) -> Result<SearchResponse>;
}

/// 搜索请求参数
pub struct SearchParams {
    pub query: String,
    pub count: usize,
    pub freshness: Option<String>,
}

/// 搜索响应
pub struct SearchResponse {
    pub query: String,
    pub provider: String,
    pub items: Vec<SearchItem>,
    pub elapsed_ms: u64,
}

/// 单条搜索结果
pub struct SearchItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// 根据 api_format 和配置创建对应的 SearchProvider
pub fn create_provider(
    api_format: &str,
    base_url: &str,
    api_key: &str,
    model_name: &str,
) -> Result<Box<dyn SearchProvider>> {
    match api_format {
        "search_brave" => Ok(Box::new(brave::BraveSearch::new(base_url, api_key))),
        "search_tavily" => Ok(Box::new(tavily::TavilySearch::new(base_url, api_key))),
        "search_metaso" => Ok(Box::new(metaso::MetasoSearch::new(base_url, api_key))),
        "search_bocha" => Ok(Box::new(bocha::BochaSearch::new(base_url, api_key))),
        "search_serpapi" => Ok(Box::new(serpapi::SerpApiSearch::new(
            base_url, api_key, model_name,
        ))),
        _ => anyhow::bail!("未知的搜索 Provider: {}", api_format),
    }
}
```

注意: 此步骤中 5 个子模块文件尚未创建，先用空的 placeholder 文件以通过编译。每个 Provider 模块在后续 Task 中逐个实现。

**Step 2: 创建 5 个 placeholder 文件**

为每个 Provider 创建最小可编译文件:

```rust
// src/agent/tools/search_providers/brave.rs
use super::{SearchParams, SearchProvider, SearchResponse};
use anyhow::Result;

pub struct BraveSearch {
    base_url: String,
    api_key: String,
}

impl BraveSearch {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for BraveSearch {
    fn name(&self) -> &str { "brave" }
    fn display_name(&self) -> &str { "Brave Search" }
    fn search(&self, _params: &SearchParams) -> Result<SearchResponse> {
        anyhow::bail!("Brave Search 尚未实现")
    }
}
```

（tavily.rs, metaso.rs, bocha.rs, serpapi.rs 用同样模板，替换名称即可。serpapi.rs 多一个 `engine` 字段。）

**Step 3: 在 tools/mod.rs 中添加模块声明**

在 `src/agent/tools/mod.rs` 第 1 行前面添加:

```rust
pub mod search_providers;
```

**Step 4: 编译验证**

Run: `cargo check`
Expected: 编译通过，无错误

**Step 5: Commit**

```bash
git add src/agent/tools/search_providers/
git add src/agent/tools/mod.rs
git commit -m "feat(search): SearchProvider trait 和公共类型定义"
```

---

## Task 2: SearchCache 内存缓存

**Files:**
- Create: `src/agent/tools/search_providers/cache.rs`
- Modify: `src/agent/tools/search_providers/mod.rs` — 添加 `pub mod cache;`

**Step 1: 写 cache 单元测试**

测试写在 `cache.rs` 底部的 `#[cfg(test)] mod tests` 中:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_hit() {
        let cache = SearchCache::new(Duration::from_secs(60), 10);
        let items = vec![SearchItem {
            title: "Test".into(),
            url: "https://example.com".into(),
            snippet: "snippet".into(),
        }];
        cache.put("brave", "rust lang", 5, items.clone());
        let result = cache.get("brave", "rust lang", 5);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
        assert_eq!(result.unwrap()[0].title, "Test");
    }

    #[test]
    fn test_cache_miss_different_query() {
        let cache = SearchCache::new(Duration::from_secs(60), 10);
        cache.put("brave", "rust", 5, vec![]);
        assert!(cache.get("brave", "python", 5).is_none());
    }

    #[test]
    fn test_cache_miss_different_provider() {
        let cache = SearchCache::new(Duration::from_secs(60), 10);
        cache.put("brave", "rust", 5, vec![]);
        assert!(cache.get("tavily", "rust", 5).is_none());
    }

    #[test]
    fn test_cache_expiry() {
        let cache = SearchCache::new(Duration::from_millis(50), 10);
        cache.put("brave", "rust", 5, vec![]);
        std::thread::sleep(Duration::from_millis(100));
        assert!(cache.get("brave", "rust", 5).is_none());
    }

    #[test]
    fn test_cache_max_size_eviction() {
        let cache = SearchCache::new(Duration::from_secs(60), 2);
        cache.put("a", "q1", 5, vec![]);
        cache.put("b", "q2", 5, vec![]);
        cache.put("c", "q3", 5, vec![]);  // 应驱逐 q1
        assert!(cache.get("a", "q1", 5).is_none());
        assert!(cache.get("b", "q2", 5).is_some());
        assert!(cache.get("c", "q3", 5).is_some());
    }

    #[test]
    fn test_cache_key_case_insensitive() {
        let cache = SearchCache::new(Duration::from_secs(60), 10);
        cache.put("brave", "Rust Lang", 5, vec![]);
        assert!(cache.get("brave", "rust lang", 5).is_some());
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test --lib agent::tools::search_providers::cache -- --nocapture`
Expected: 编译错误（`SearchCache` 未定义）

**Step 3: 实现 SearchCache**

```rust
// src/agent/tools/search_providers/cache.rs

use super::SearchItem;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct CacheEntry {
    items: Vec<SearchItem>,
    created_at: Instant,
}

pub struct SearchCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    ttl: Duration,
    max_size: usize,
}

impl SearchCache {
    pub fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            ttl,
            max_size,
        }
    }

    /// 生成缓存 key："{provider}:{query_lowercase}:{count}"
    fn cache_key(provider: &str, query: &str, count: usize) -> String {
        format!("{}:{}:{}", provider, query.to_lowercase(), count)
    }

    pub fn get(&self, provider: &str, query: &str, count: usize) -> Option<Vec<SearchItem>> {
        let key = Self::cache_key(provider, query, count);
        let mut entries = self.entries.lock().unwrap();

        // 清理过期条目
        entries.retain(|_, v| v.created_at.elapsed() < self.ttl);

        entries.get(&key).map(|e| e.items.clone())
    }

    pub fn put(&self, provider: &str, query: &str, count: usize, items: Vec<SearchItem>) {
        let key = Self::cache_key(provider, query, count);
        let mut entries = self.entries.lock().unwrap();

        // 清理过期条目
        entries.retain(|_, v| v.created_at.elapsed() < self.ttl);

        // 超过 max_size 时移除最旧条目
        while entries.len() >= self.max_size {
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, v)| v.created_at)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
            } else {
                break;
            }
        }

        entries.insert(key, CacheEntry {
            items,
            created_at: Instant::now(),
        });
    }
}
```

注意: `SearchItem` 需要 `#[derive(Clone)]`。回到 `mod.rs` 给 `SearchItem` 加上 `Clone` derive。

**Step 4: 在 mod.rs 中添加模块声明和 Clone derive**

在 `mod.rs` 添加 `pub mod cache;`，并为 `SearchItem` 添加 `#[derive(Clone)]`。

**Step 5: 运行测试验证通过**

Run: `cargo test --lib agent::tools::search_providers::cache -- --nocapture`
Expected: 6 tests passed

**Step 6: Commit**

```bash
git add src/agent/tools/search_providers/cache.rs
git add src/agent/tools/search_providers/mod.rs
git commit -m "feat(search): SearchCache 内存缓存实现"
```

---

## Task 3: Brave Search Provider

**Files:**
- Modify: `src/agent/tools/search_providers/brave.rs`

**Step 1: 写 Brave 解析测试**

使用 mock JSON 测试响应解析逻辑（不需要真实 API Key）:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_brave_response() {
        let json = serde_json::json!({
            "web": {
                "results": [
                    {
                        "title": "Rust 编程语言",
                        "url": "https://www.rust-lang.org/",
                        "description": "一种注重安全和性能的系统编程语言"
                    },
                    {
                        "title": "Rust - Wikipedia",
                        "url": "https://en.wikipedia.org/wiki/Rust",
                        "description": "Rust is a multi-paradigm programming language"
                    }
                ]
            }
        });
        let items = parse_brave_response(&json);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Rust 编程语言");
        assert_eq!(items[0].url, "https://www.rust-lang.org/");
        assert_eq!(items[1].title, "Rust - Wikipedia");
    }

    #[test]
    fn test_parse_brave_empty_response() {
        let json = serde_json::json!({});
        let items = parse_brave_response(&json);
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_brave_no_web_results() {
        let json = serde_json::json!({ "web": {} });
        let items = parse_brave_response(&json);
        assert!(items.is_empty());
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test --lib agent::tools::search_providers::brave -- --nocapture`
Expected: 编译失败（`parse_brave_response` 未定义）

**Step 3: 实现 Brave Search**

```rust
// src/agent/tools/search_providers/brave.rs

use super::{SearchItem, SearchParams, SearchProvider, SearchResponse};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::time::Instant;

pub struct BraveSearch {
    base_url: String,
    api_key: String,
}

impl BraveSearch {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for BraveSearch {
    fn name(&self) -> &str { "brave" }
    fn display_name(&self) -> &str { "Brave Search" }

    fn search(&self, params: &SearchParams) -> Result<SearchResponse> {
        let start = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let mut url = format!(
            "{}/res/v1/web/search?q={}&count={}",
            self.base_url,
            urlencoding::encode(&params.query),
            params.count
        );
        if let Some(ref freshness) = params.freshness {
            url.push_str(&format!("&freshness={}", freshness));
        }

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .map_err(|e| anyhow!("Brave 搜索请求失败: {}", e))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(anyhow!("搜索配置错误: API Key 无效，请在设置中检查"));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!("搜索频率超限: 请稍后重试"));
        }
        if status.is_server_error() {
            return Err(anyhow!("搜索服务暂不可用: Brave Search"));
        }
        if !status.is_success() {
            return Err(anyhow!("Brave 搜索返回错误: {}", status));
        }

        let body: Value = resp.json()
            .map_err(|e| anyhow!("Brave 响应解析失败: {}", e))?;

        let items = parse_brave_response(&body);

        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "brave".to_string(),
            items,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

fn parse_brave_response(json: &Value) -> Vec<SearchItem> {
    json["web"]["results"]
        .as_array()
        .map(|results| {
            results.iter().filter_map(|r| {
                Some(SearchItem {
                    title: r["title"].as_str()?.to_string(),
                    url: r["url"].as_str()?.to_string(),
                    snippet: r["description"].as_str().unwrap_or("").to_string(),
                })
            }).collect()
        })
        .unwrap_or_default()
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test --lib agent::tools::search_providers::brave -- --nocapture`
Expected: 3 tests passed

**Step 5: Commit**

```bash
git add src/agent/tools/search_providers/brave.rs
git commit -m "feat(search): Brave Search Provider 实现"
```

---

## Task 4: Tavily Search Provider

**Files:**
- Modify: `src/agent/tools/search_providers/tavily.rs`

**Step 1: 写 Tavily 解析测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tavily_response() {
        let json = serde_json::json!({
            "results": [
                {
                    "title": "Rust 入门",
                    "url": "https://doc.rust-lang.org/book/",
                    "content": "Rust 官方入门教程"
                }
            ],
            "answer": "Rust 是一种系统编程语言"
        });
        let items = parse_tavily_response(&json);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Rust 入门");
        assert_eq!(items[0].snippet, "Rust 官方入门教程");
    }

    #[test]
    fn test_parse_tavily_empty() {
        let json = serde_json::json!({});
        let items = parse_tavily_response(&json);
        assert!(items.is_empty());
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test --lib agent::tools::search_providers::tavily -- --nocapture`
Expected: 编译失败

**Step 3: 实现 Tavily Search**

```rust
// src/agent/tools/search_providers/tavily.rs

use super::{SearchItem, SearchParams, SearchProvider, SearchResponse};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Instant;

pub struct TavilySearch {
    base_url: String,
    api_key: String,
}

impl TavilySearch {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for TavilySearch {
    fn name(&self) -> &str { "tavily" }
    fn display_name(&self) -> &str { "Tavily" }

    fn search(&self, params: &SearchParams) -> Result<SearchResponse> {
        let start = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let body = json!({
            "query": params.query,
            "max_results": params.count,
            "search_depth": "basic",
            "include_answer": false,
        });

        let resp = client
            .post(&format!("{}/search", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .map_err(|e| anyhow!("Tavily 搜索请求失败: {}", e))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(anyhow!("搜索配置错误: API Key 无效，请在设置中检查"));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!("搜索频率超限: 请稍后重试"));
        }
        if status.is_server_error() {
            return Err(anyhow!("搜索服务暂不可用: Tavily"));
        }
        if !status.is_success() {
            return Err(anyhow!("Tavily 搜索返回错误: {}", status));
        }

        let resp_json: Value = resp.json()
            .map_err(|e| anyhow!("Tavily 响应解析失败: {}", e))?;

        let items = parse_tavily_response(&resp_json);

        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "tavily".to_string(),
            items,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

fn parse_tavily_response(json: &Value) -> Vec<SearchItem> {
    json["results"]
        .as_array()
        .map(|results| {
            results.iter().filter_map(|r| {
                Some(SearchItem {
                    title: r["title"].as_str()?.to_string(),
                    url: r["url"].as_str()?.to_string(),
                    snippet: r["content"].as_str().unwrap_or("").to_string(),
                })
            }).collect()
        })
        .unwrap_or_default()
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test --lib agent::tools::search_providers::tavily -- --nocapture`
Expected: 2 tests passed

**Step 5: Commit**

```bash
git add src/agent/tools/search_providers/tavily.rs
git commit -m "feat(search): Tavily Search Provider 实现"
```

---

## Task 5: 博查搜索 Provider

**Files:**
- Modify: `src/agent/tools/search_providers/bocha.rs`

**Step 1: 写博查解析测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bocha_response() {
        let json = serde_json::json!({
            "data": {
                "webPages": {
                    "value": [
                        {
                            "name": "Rust 语言",
                            "url": "https://rust-lang.org",
                            "snippet": "系统编程语言"
                        }
                    ]
                }
            }
        });
        let items = parse_bocha_response(&json);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Rust 语言");
    }

    #[test]
    fn test_parse_bocha_empty() {
        let json = serde_json::json!({});
        let items = parse_bocha_response(&json);
        assert!(items.is_empty());
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test --lib agent::tools::search_providers::bocha -- --nocapture`
Expected: 编译失败

**Step 3: 实现博查搜索**

```rust
// src/agent/tools/search_providers/bocha.rs

use super::{SearchItem, SearchParams, SearchProvider, SearchResponse};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Instant;

pub struct BochaSearch {
    base_url: String,
    api_key: String,
}

impl BochaSearch {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for BochaSearch {
    fn name(&self) -> &str { "bocha" }
    fn display_name(&self) -> &str { "博查搜索" }

    fn search(&self, params: &SearchParams) -> Result<SearchResponse> {
        let start = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let mut body = json!({
            "query": params.query,
            "count": params.count,
        });
        if let Some(ref freshness) = params.freshness {
            body["freshness"] = Value::String(freshness.clone());
        }

        let resp = client
            .post(&format!("{}/v1/web-search", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .map_err(|e| anyhow!("博查搜索请求失败: {}", e))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(anyhow!("搜索配置错误: API Key 无效，请在设置中检查"));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!("搜索频率超限: 请稍后重试"));
        }
        if status.is_server_error() {
            return Err(anyhow!("搜索服务暂不可用: 博查搜索"));
        }
        if !status.is_success() {
            return Err(anyhow!("博查搜索返回错误: {}", status));
        }

        let resp_json: Value = resp.json()
            .map_err(|e| anyhow!("博查响应解析失败: {}", e))?;

        let items = parse_bocha_response(&resp_json);

        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "bocha".to_string(),
            items,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

fn parse_bocha_response(json: &Value) -> Vec<SearchItem> {
    json["data"]["webPages"]["value"]
        .as_array()
        .map(|results| {
            results.iter().filter_map(|r| {
                Some(SearchItem {
                    title: r["name"].as_str()?.to_string(),
                    url: r["url"].as_str()?.to_string(),
                    snippet: r["snippet"].as_str().unwrap_or("").to_string(),
                })
            }).collect()
        })
        .unwrap_or_default()
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test --lib agent::tools::search_providers::bocha -- --nocapture`
Expected: 2 tests passed

**Step 5: Commit**

```bash
git add src/agent/tools/search_providers/bocha.rs
git commit -m "feat(search): 博查搜索 Provider 实现"
```

---

## Task 6: SerpAPI Provider

**Files:**
- Modify: `src/agent/tools/search_providers/serpapi.rs`

**Step 1: 写 SerpAPI 解析测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_serpapi_response() {
        let json = serde_json::json!({
            "organic_results": [
                {
                    "title": "Rust Programming",
                    "link": "https://rust-lang.org",
                    "snippet": "Systems language"
                }
            ]
        });
        let items = parse_serpapi_response(&json);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "https://rust-lang.org");
    }

    #[test]
    fn test_parse_serpapi_empty() {
        let json = serde_json::json!({});
        let items = parse_serpapi_response(&json);
        assert!(items.is_empty());
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test --lib agent::tools::search_providers::serpapi -- --nocapture`
Expected: 编译失败

**Step 3: 实现 SerpAPI**

```rust
// src/agent/tools/search_providers/serpapi.rs

use super::{SearchItem, SearchParams, SearchProvider, SearchResponse};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::time::Instant;

pub struct SerpApiSearch {
    base_url: String,
    api_key: String,
    engine: String,
}

impl SerpApiSearch {
    pub fn new(base_url: &str, api_key: &str, engine: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            engine: if engine.is_empty() { "google".to_string() } else { engine.to_string() },
        }
    }
}

impl SearchProvider for SerpApiSearch {
    fn name(&self) -> &str { "serpapi" }
    fn display_name(&self) -> &str { "SerpAPI" }

    fn search(&self, params: &SearchParams) -> Result<SearchResponse> {
        let start = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let url = format!(
            "{}/search.json?engine={}&q={}&num={}&api_key={}",
            self.base_url,
            urlencoding::encode(&self.engine),
            urlencoding::encode(&params.query),
            params.count,
            urlencoding::encode(&self.api_key),
        );

        let resp = client
            .get(&url)
            .send()
            .map_err(|e| anyhow!("SerpAPI 搜索请求失败: {}", e))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(anyhow!("搜索配置错误: API Key 无效，请在设置中检查"));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!("搜索频率超限: 请稍后重试"));
        }
        if status.is_server_error() {
            return Err(anyhow!("搜索服务暂不可用: SerpAPI"));
        }
        if !status.is_success() {
            return Err(anyhow!("SerpAPI 搜索返回错误: {}", status));
        }

        let resp_json: Value = resp.json()
            .map_err(|e| anyhow!("SerpAPI 响应解析失败: {}", e))?;

        let items = parse_serpapi_response(&resp_json);

        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "serpapi".to_string(),
            items,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

fn parse_serpapi_response(json: &Value) -> Vec<SearchItem> {
    json["organic_results"]
        .as_array()
        .map(|results| {
            results.iter().filter_map(|r| {
                Some(SearchItem {
                    title: r["title"].as_str()?.to_string(),
                    url: r["link"].as_str()?.to_string(),
                    snippet: r["snippet"].as_str().unwrap_or("").to_string(),
                })
            }).collect()
        })
        .unwrap_or_default()
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test --lib agent::tools::search_providers::serpapi -- --nocapture`
Expected: 2 tests passed

**Step 5: Commit**

```bash
git add src/agent/tools/search_providers/serpapi.rs
git commit -m "feat(search): SerpAPI Provider 实现"
```

---

## Task 7: 秘塔搜索 Provider

秘塔搜索 API 可能返回 streaming SSE 或普通 JSON。由于其 API 文档有限，实现时按普通 JSON POST 处理。如实际 API 为 SSE 格式，后续调整。

**Files:**
- Modify: `src/agent/tools/search_providers/metaso.rs`

**Step 1: 写秘塔解析测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metaso_response() {
        let json = serde_json::json!({
            "data": {
                "items": [
                    {
                        "title": "Rust 编程",
                        "url": "https://rust-lang.org",
                        "content": "系统编程语言"
                    }
                ]
            }
        });
        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Rust 编程");
    }

    #[test]
    fn test_parse_metaso_empty() {
        let json = serde_json::json!({});
        let items = parse_metaso_response(&json);
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_metaso_alternative_format() {
        // 秘塔可能返回不同格式，验证兼容性
        let json = serde_json::json!({
            "results": [
                {
                    "title": "Test",
                    "url": "https://example.com",
                    "snippet": "A test result"
                }
            ]
        });
        let items = parse_metaso_response(&json);
        assert_eq!(items.len(), 1);
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test --lib agent::tools::search_providers::metaso -- --nocapture`
Expected: 编译失败

**Step 3: 实现秘塔搜索**

```rust
// src/agent/tools/search_providers/metaso.rs

use super::{SearchItem, SearchParams, SearchProvider, SearchResponse};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Instant;

pub struct MetasoSearch {
    base_url: String,
    api_key: String,
}

impl MetasoSearch {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }
}

impl SearchProvider for MetasoSearch {
    fn name(&self) -> &str { "metaso" }
    fn display_name(&self) -> &str { "秘塔搜索" }

    fn search(&self, params: &SearchParams) -> Result<SearchResponse> {
        let start = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let body = json!({
            "query": params.query,
            "mode": "concise",
        });

        let resp = client
            .post(&format!("{}/api/search", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .map_err(|e| anyhow!("秘塔搜索请求失败: {}", e))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(anyhow!("搜索配置错误: API Key 无效，请在设置中检查"));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!("搜索频率超限: 请稍后重试"));
        }
        if status.is_server_error() {
            return Err(anyhow!("搜索服务暂不可用: 秘塔搜索"));
        }
        if !status.is_success() {
            return Err(anyhow!("秘塔搜索返回错误: {}", status));
        }

        let resp_json: Value = resp.json()
            .map_err(|e| anyhow!("秘塔响应解析失败: {}", e))?;

        let items = parse_metaso_response(&resp_json);

        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "metaso".to_string(),
            items,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// 解析秘塔响应，兼容多种可能的 JSON 格式
fn parse_metaso_response(json: &Value) -> Vec<SearchItem> {
    // 格式 1: { data: { items: [...] } }
    if let Some(items) = json["data"]["items"].as_array() {
        let parsed: Vec<SearchItem> = items.iter().filter_map(|r| {
            Some(SearchItem {
                title: r["title"].as_str()?.to_string(),
                url: r["url"].as_str()?.to_string(),
                snippet: r["content"].as_str()
                    .or(r["snippet"].as_str())
                    .unwrap_or("").to_string(),
            })
        }).collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    // 格式 2: { results: [...] }
    if let Some(results) = json["results"].as_array() {
        return results.iter().filter_map(|r| {
            Some(SearchItem {
                title: r["title"].as_str()?.to_string(),
                url: r["url"].as_str()?.to_string(),
                snippet: r["snippet"].as_str()
                    .or(r["content"].as_str())
                    .unwrap_or("").to_string(),
            })
        }).collect();
    }
    Vec::new()
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test --lib agent::tools::search_providers::metaso -- --nocapture`
Expected: 3 tests passed

**Step 5: Commit**

```bash
git add src/agent/tools/search_providers/metaso.rs
git commit -m "feat(search): 秘塔搜索 Provider 实现"
```

---

## Task 8: 重构 WebSearchTool 使用 Provider

**Files:**
- Modify: `src/agent/tools/web_search.rs` — 完全重写
- Modify: `src/agent/tools/mod.rs` — 更新 re-export

**Step 1: 写 WebSearchTool 单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::tools::search_providers::{
        SearchItem, SearchParams, SearchProvider, SearchResponse,
    };
    use std::sync::Arc;
    use std::time::Duration;

    /// 用于测试的 mock provider
    struct MockProvider;
    impl SearchProvider for MockProvider {
        fn name(&self) -> &str { "mock" }
        fn display_name(&self) -> &str { "Mock Search" }
        fn search(&self, params: &SearchParams) -> anyhow::Result<SearchResponse> {
            Ok(SearchResponse {
                query: params.query.clone(),
                provider: "mock".to_string(),
                items: vec![
                    SearchItem {
                        title: "Mock Result".to_string(),
                        url: "https://example.com".to_string(),
                        snippet: "A mock result".to_string(),
                    },
                ],
                elapsed_ms: 10,
            })
        }
    }

    #[test]
    fn test_web_search_with_provider() {
        let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
        let tool = WebSearchTool::with_provider(Box::new(MockProvider), cache);
        let result = tool.execute(
            serde_json::json!({"query": "test"}),
            &ToolContext::default(),
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Mock Result"));
        assert!(output.contains("https://example.com"));
    }

    #[test]
    fn test_web_search_missing_query() {
        let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
        let tool = WebSearchTool::with_provider(Box::new(MockProvider), cache);
        let result = tool.execute(serde_json::json!({}), &ToolContext::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_web_search_empty_query() {
        let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
        let tool = WebSearchTool::with_provider(Box::new(MockProvider), cache);
        let result = tool.execute(
            serde_json::json!({"query": "  "}),
            &ToolContext::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_web_search_uses_cache() {
        let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
        let tool = WebSearchTool::with_provider(Box::new(MockProvider), cache.clone());

        // 第一次调用：应缓存结果
        let _ = tool.execute(
            serde_json::json!({"query": "cached test"}),
            &ToolContext::default(),
        );

        // 验证缓存命中
        assert!(cache.get("mock", "cached test", 5).is_some());
    }

    #[test]
    fn test_web_search_freshness_param() {
        let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
        let tool = WebSearchTool::with_provider(Box::new(MockProvider), cache);
        let result = tool.execute(
            serde_json::json!({"query": "news", "freshness": "day"}),
            &ToolContext::default(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_web_search_result_truncation() {
        // 结果截断到 30,000 字符
        struct LongProvider;
        impl SearchProvider for LongProvider {
            fn name(&self) -> &str { "long" }
            fn display_name(&self) -> &str { "Long" }
            fn search(&self, params: &SearchParams) -> anyhow::Result<SearchResponse> {
                let long_snippet = "x".repeat(40000);
                Ok(SearchResponse {
                    query: params.query.clone(),
                    provider: "long".to_string(),
                    items: vec![SearchItem {
                        title: "Long".to_string(),
                        url: "https://example.com".to_string(),
                        snippet: long_snippet,
                    }],
                    elapsed_ms: 1,
                })
            }
        }
        let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
        let tool = WebSearchTool::with_provider(Box::new(LongProvider), cache);
        let result = tool.execute(
            serde_json::json!({"query": "long"}),
            &ToolContext::default(),
        ).unwrap();
        assert!(result.len() <= 30100); // 30000 + 一些截断提示文字
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test --lib agent::tools::web_search -- --nocapture`
Expected: 编译失败（新 API 不存在）

**Step 3: 重写 WebSearchTool**

```rust
// src/agent/tools/web_search.rs

use crate::agent::tools::search_providers::{
    SearchParams, SearchProvider,
    cache::SearchCache,
};
use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

const MAX_OUTPUT_CHARS: usize = 30_000;

pub struct WebSearchTool {
    provider: Box<dyn SearchProvider>,
    cache: Arc<SearchCache>,
}

impl WebSearchTool {
    /// 使用指定的搜索 Provider 和缓存创建
    pub fn with_provider(provider: Box<dyn SearchProvider>, cache: Arc<SearchCache>) -> Self {
        Self { provider, cache }
    }
}

impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "搜索互联网获取最新信息。返回网页标题、URL 和摘要。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索关键词"
                },
                "count": {
                    "type": "integer",
                    "description": "返回结果数量 (1-10，默认 5)",
                    "default": 5
                },
                "freshness": {
                    "type": "string",
                    "description": "结果新鲜度: day/week/month/year",
                    "enum": ["day", "week", "month", "year"]
                }
            },
            "required": ["query"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        let query = input["query"]
            .as_str()
            .ok_or(anyhow!("缺少 query 参数"))?;

        if query.trim().is_empty() {
            return Err(anyhow!("query 不能为空"));
        }

        let count = input["count"].as_i64().unwrap_or(5).clamp(1, 10) as usize;
        let freshness = input["freshness"].as_str().map(String::from);

        // 检查缓存（freshness 请求不缓存）
        if freshness.is_none() {
            if let Some(cached_items) = self.cache.get(self.provider.name(), query, count) {
                let output = format_results(&cached_items, self.provider.display_name());
                return Ok(truncate_output(&output));
            }
        }

        let params = SearchParams {
            query: query.to_string(),
            count,
            freshness,
        };

        let response = self.provider.search(&params)?;

        if response.items.is_empty() {
            return Ok("未找到搜索结果".to_string());
        }

        // 写入缓存
        self.cache.put(
            self.provider.name(),
            query,
            count,
            response.items.clone(),
        );

        let output = format_results(&response.items, self.provider.display_name());
        Ok(truncate_output(&output))
    }
}

use crate::agent::tools::search_providers::SearchItem;

fn format_results(items: &[SearchItem], provider_name: &str) -> String {
    let mut output = format!("[搜索结果 - 来自 {}]\n\n", provider_name);
    for (i, item) in items.iter().enumerate() {
        if item.snippet.is_empty() {
            output.push_str(&format!("{}. {}\n   {}\n\n", i + 1, item.title, item.url));
        } else {
            output.push_str(&format!(
                "{}. {}\n   {}\n   {}\n\n",
                i + 1, item.title, item.url, item.snippet
            ));
        }
    }
    output
}

fn truncate_output(output: &str) -> String {
    if output.len() > MAX_OUTPUT_CHARS {
        let truncated = &output[..MAX_OUTPUT_CHARS];
        format!("{}\n\n... (结果已截断)", truncated)
    } else {
        output.to_string()
    }
}
```

**Step 4: 更新 mod.rs 的 re-export**

`WebSearchTool` 的构造方法变了（不再有 `new()` 和 `standalone()`），需要确保 `mod.rs` 的 `pub use` 仍然正确。无需改动，`pub use web_search::WebSearchTool;` 已存在。

**Step 5: 运行测试验证通过**

Run: `cargo test --lib agent::tools::web_search -- --nocapture`
Expected: 6 tests passed

**Step 6: Commit**

```bash
git add src/agent/tools/web_search.rs
git commit -m "feat(search): WebSearchTool 重构为 Provider 模式"
```

---

## Task 9: chat.rs 集成 — 从 DB 动态加载搜索 Provider

**Files:**
- Modify: `src/commands/chat.rs` — 替换 WebSearchTool 注册逻辑

**Step 1: 修改 chat.rs 中 WebSearchTool 注册部分**

找到当前代码（约第 227-229 行）:

```rust
// 注册 WebSearch 工具（通过 Sidecar 代理）
let web_search = WebSearchTool::new("http://localhost:8765".to_string());
agent_executor.registry().register(Arc::new(web_search));
```

替换为:

```rust
// 注册 WebSearch 工具（从 DB 加载搜索 Provider 配置）
{
    use crate::agent::tools::search_providers::{cache::SearchCache, create_provider};
    use std::time::Duration;

    // 全局搜索缓存（lazy_static 或 once_cell）
    // 这里每次 send_message 创建新缓存实例也可以（按 session 级别缓存）
    let search_cache = Arc::new(SearchCache::new(Duration::from_secs(900), 100));

    let search_config = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT api_format, base_url, api_key, model_name FROM model_configs WHERE api_format LIKE 'search_%' AND is_default = 1 LIMIT 1"
    )
    .fetch_optional(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    if let Some((api_format, base_url, search_api_key, search_model_name)) = search_config {
        match create_provider(&api_format, &base_url, &search_api_key, &search_model_name) {
            Ok(provider) => {
                let web_search = WebSearchTool::with_provider(provider, search_cache);
                agent_executor.registry().register(Arc::new(web_search));
            }
            Err(e) => {
                eprintln!("[search] 创建搜索 Provider 失败: {}", e);
            }
        }
    }
    // 无搜索配置时不注册 web_search 工具，Agent 不调用搜索
}
```

**Step 2: 更新 import 语句**

从 import 中移除 `WebSearchTool` 的直接导入（因为现在在内部块中使用），或者保留 import 但更新用法。

实际上 `WebSearchTool` 仍然需要 import，因为我们在块内使用它。保留现有 import 即可。

**Step 3: 编译验证**

Run: `cargo check`
Expected: 编译通过

**Step 4: Commit**

```bash
git add src/commands/chat.rs
git commit -m "feat(search): chat.rs 从 DB 动态加载搜索 Provider"
```

---

## Task 10: 后端搜索配置 CRUD 命令

**Files:**
- Modify: `src/commands/models.rs` — 添加搜索配置的列表/测试命令
- Modify: `src/lib.rs` — 注册新命令

现有的 `save_model_config`、`delete_model_config` 已经适用于搜索配置（因为复用 model_configs 表）。需要添加:
1. `list_search_configs` — 列出搜索 Provider 配置
2. `test_search_connection` — 测试搜索 Provider 连接

**Step 1: 在 models.rs 中添加搜索配置命令**

```rust
/// 列出所有搜索 Provider 配置
#[tauri::command]
pub async fn list_search_configs(db: State<'_, DbState>) -> Result<Vec<ModelConfig>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, bool)>(
        "SELECT id, name, api_format, base_url, model_name, CAST(is_default AS BOOLEAN) FROM model_configs WHERE api_format LIKE 'search_%'"
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name, api_format, base_url, model_name, is_default)| {
        ModelConfig { id, name, api_format, base_url, model_name, is_default }
    }).collect())
}

/// 测试搜索 Provider 连接
#[tauri::command]
pub async fn test_search_connection(config: ModelConfig, api_key: String) -> Result<bool, String> {
    use crate::agent::tools::search_providers::{create_provider, SearchParams};

    let provider = create_provider(&config.api_format, &config.base_url, &api_key, &config.model_name)
        .map_err(|e| format!("创建 Provider 失败: {}", e))?;

    // 用简单查询测试连接
    let result = tokio::task::spawn_blocking(move || {
        provider.search(&SearchParams {
            query: "test".to_string(),
            count: 1,
            freshness: None,
        })
    })
    .await
    .map_err(|e| format!("测试线程异常: {}", e))?;

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("连接测试失败: {}", e)),
    }
}

/// 设置默认搜索 Provider（同时取消其他搜索配置的默认状态）
#[tauri::command]
pub async fn set_default_search(config_id: String, db: State<'_, DbState>) -> Result<(), String> {
    // 先取消所有搜索配置的默认状态
    sqlx::query("UPDATE model_configs SET is_default = 0 WHERE api_format LIKE 'search_%'")
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    // 设置指定配置为默认
    sqlx::query("UPDATE model_configs SET is_default = 1 WHERE id = ?")
        .bind(&config_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

**Step 2: 在 lib.rs 中注册新命令**

找到 `tauri::Builder` 的 `.invoke_handler(tauri::generate_handler![...])` 部分，添加:

```rust
commands::models::list_search_configs,
commands::models::test_search_connection,
commands::models::set_default_search,
```

**Step 3: 编译验证**

Run: `cargo check`
Expected: 编译通过

**Step 4: Commit**

```bash
git add src/commands/models.rs src/lib.rs
git commit -m "feat(search): 搜索配置 CRUD 命令 (list/test/set_default)"
```

---

## Task 11: 前端搜索引擎设置页面

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx` — 添加"搜索引擎"Tab

**Step 1: 添加搜索引擎 Provider 预设常量**

在文件顶部 `PROVIDER_PRESETS` 后添加:

```typescript
const SEARCH_PRESETS = [
  { label: "— 快速选择 —", value: "", api_format: "", base_url: "", model_name: "" },
  { label: "Brave Search (国际首选)", value: "brave", api_format: "search_brave", base_url: "https://api.search.brave.com", model_name: "" },
  { label: "Tavily (AI 专用)", value: "tavily", api_format: "search_tavily", base_url: "https://api.tavily.com", model_name: "" },
  { label: "秘塔搜索 (中文首选)", value: "metaso", api_format: "search_metaso", base_url: "https://api.metaso.cn", model_name: "" },
  { label: "博查搜索 (中文 AI)", value: "bocha", api_format: "search_bocha", base_url: "https://api.bochaai.com", model_name: "" },
  { label: "SerpAPI (多引擎)", value: "serpapi", api_format: "search_serpapi", base_url: "https://serpapi.com", model_name: "google" },
];
```

**Step 2: 添加搜索引擎 Tab 状态和功能**

在 `SettingsView` 组件中:
- 扩展 `activeTab` 类型为 `"models" | "mcp" | "search"`
- 添加 `searchConfigs` state 和搜索表单 state
- 添加 `loadSearchConfigs()`、`handleSaveSearch()`、`handleTestSearch()`、`handleSetDefault()` 函数

**Step 3: 添加搜索引擎 Tab UI**

Tab 切换栏增加"搜索引擎"按钮。搜索引擎配置面板包含:
- 已配置的搜索引擎列表（显示名称、Provider 类型、默认标记）
- Provider 快速选择下拉
- API Key 输入框
- 可选自定义 Base URL
- 测试连接 / 保存 / 设为默认按钮

**Step 4: 编译验证**

Run: `pnpm --filter runtime build` (或 `pnpm runtime` 启动开发模式)
Expected: 前端编译通过，设置页面显示三个 Tab

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx
git commit -m "feat(search): 前端搜索引擎配置页面"
```

---

## Task 12: 全局搜索缓存单例 + 集成测试

**Files:**
- Modify: `src/lib.rs` — 添加全局 SearchCache state
- Modify: `src/commands/chat.rs` — 使用全局缓存 state
- Create: `tests/test_search_providers.rs` — 集成测试

**Step 1: 在 lib.rs 中添加全局 SearchCache**

```rust
use crate::agent::tools::search_providers::cache::SearchCache;

// 在 Tauri builder 的 setup 闭包中:
let search_cache = Arc::new(SearchCache::new(
    std::time::Duration::from_secs(900), // 15 分钟 TTL
    100, // 最多 100 条
));
app.manage(SearchCacheState(search_cache));

// State 包装
pub struct SearchCacheState(pub Arc<SearchCache>);
```

**Step 2: 修改 chat.rs 使用全局缓存**

在 `send_message` 中:

```rust
let search_cache = app.state::<crate::commands::chat::SearchCacheState>().0.clone();
```

替换之前每次新建的 `SearchCache::new(...)`。

**Step 3: 写集成测试**

```rust
// tests/test_search_providers.rs

use runtime_lib::agent::tools::search_providers::{
    SearchParams, SearchProvider,
    brave::BraveSearch,
    tavily::TavilySearch,
    bocha::BochaSearch,
    serpapi::SerpApiSearch,
    metaso::MetasoSearch,
    cache::SearchCache,
    create_provider,
};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_create_provider_brave() {
    let provider = create_provider("search_brave", "https://api.search.brave.com", "test_key", "");
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().name(), "brave");
}

#[test]
fn test_create_provider_tavily() {
    let provider = create_provider("search_tavily", "https://api.tavily.com", "test_key", "");
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().name(), "tavily");
}

#[test]
fn test_create_provider_bocha() {
    let provider = create_provider("search_bocha", "https://api.bochaai.com", "test_key", "");
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().name(), "bocha");
}

#[test]
fn test_create_provider_serpapi() {
    let provider = create_provider("search_serpapi", "https://serpapi.com", "test_key", "google");
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().name(), "serpapi");
}

#[test]
fn test_create_provider_metaso() {
    let provider = create_provider("search_metaso", "https://api.metaso.cn", "test_key", "");
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().name(), "metaso");
}

#[test]
fn test_create_provider_unknown() {
    let provider = create_provider("search_unknown", "", "", "");
    assert!(provider.is_err());
}

#[test]
fn test_cache_shared_across_providers() {
    let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
    let cache2 = cache.clone();

    // Provider A 写入缓存
    cache.put("brave", "rust", 5, vec![]);
    // 共享缓存可以读取
    assert!(cache2.get("brave", "rust", 5).is_some());
}
```

**Step 4: 运行所有测试**

Run: `cargo test`
Expected: 所有测试通过

**Step 5: Commit**

```bash
git add src/lib.rs src/commands/chat.rs tests/test_search_providers.rs
git commit -m "feat(search): 全局缓存单例 + 集成测试"
```

---

## Task 13: 清理旧代码 + 最终验证

**Files:**
- Modify: `src/agent/tools/web_search.rs` — 确认无 DuckDuckGo 残留
- Verify: 整个搜索系统端到端工作

**Step 1: 检查旧代码清理**

确认 `web_search.rs` 中已无:
- `parse_duckduckgo_html` 函数
- `new(_sidecar_url)` 构造函数
- `standalone()` 构造函数
- `reqwest::blocking::Client` 直接调用
- DuckDuckGo URL

**Step 2: 编译验证**

Run: `cargo check`
Expected: 编译通过

**Step 3: 运行全量测试**

Run: `cargo test`
Expected: 所有测试通过

**Step 4: 启动应用验证**

Run: `pnpm runtime`

验证:
1. 设置页面 → "搜索引擎" Tab 可见
2. 添加搜索 Provider（如有 API Key）
3. 发送搜索请求，验证结果格式正确
4. 无搜索配置时，Agent 不出现 web_search 工具

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(search): 多 Provider 网络搜索系统完成"
```
