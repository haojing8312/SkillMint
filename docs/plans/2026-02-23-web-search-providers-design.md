# 生产级多 Provider 网络搜索系统设计

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 web_search 工具从单一 DuckDuckGo HTML 解析升级为支持多个搜索 Provider 的生产级系统，包括国际和中国国内方案。

**Architecture:** Provider Trait 抽象，纯 Rust 实现，不依赖 Sidecar。通过 `model_configs` 表（`api_format` 以 `search_` 前缀标识）存储搜索配置，支持动态切换 Provider。

**Tech Stack:** Rust, reqwest, serde_json, regex, urlencoding, SQLite (sqlx)

---

## 1. SearchProvider Trait

```rust
pub trait SearchProvider: Send + Sync {
    /// Provider 标识符，如 "brave", "metaso"
    fn name(&self) -> &str;
    /// 显示名称，如 "Brave Search", "秘塔搜索"
    fn display_name(&self) -> &str;
    /// 执行搜索
    fn search(&self, params: SearchParams) -> Result<SearchResult>;
}

pub struct SearchParams {
    pub query: String,
    pub count: usize,              // 1-10，默认 5
    pub lang: Option<String>,      // "zh", "en"
    pub freshness: Option<String>, // "day", "week", "month", "year"
}

pub struct SearchResult {
    pub query: String,
    pub provider: String,
    pub items: Vec<SearchItem>,
    pub elapsed_ms: u64,
}

pub struct SearchItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}
```

## 2. Provider 列表

### 2.1 Brave Search（国际首选）

- **API**: `GET https://api.search.brave.com/res/v1/web/search`
- **认证**: Header `X-Subscription-Token: {api_key}`
- **参数**: `q`, `count`, `search_lang`, `country`, `freshness`
- **返回**: JSON `{ web: { results: [{ title, url, description }] } }`
- **api_format**: `search_brave`

### 2.2 Tavily（AI Agent 专用）

- **API**: `POST https://api.tavily.com/search`
- **认证**: Body `{ api_key: "..." }`
- **参数**: `query`, `max_results`, `search_depth`, `include_answer`
- **返回**: JSON `{ results: [{ title, url, content }], answer }`
- **api_format**: `search_tavily`
- **特点**: 专为 AI Agent 设计，返回 AI 摘要

### 2.3 秘塔搜索 Metaso（中文首选）

- **API**: `POST https://api.metaso.cn/api/search` (需确认实际端点)
- **认证**: Header `Authorization: Bearer {api_key}`
- **参数**: `query`, `mode` (concise/detail/research)
- **返回**: JSON，含搜索结果和 AI 摘要
- **api_format**: `search_metaso`

### 2.4 博查搜索 Bocha（中文 AI 搜索）

- **API**: `POST https://api.bochaai.com/v1/web-search`
- **认证**: Header `Authorization: Bearer {api_key}`
- **参数**: `query`, `count`, `freshness`
- **返回**: JSON `{ data: { webPages: { value: [{ name, url, snippet }] } } }`
- **api_format**: `search_bocha`

### 2.5 SerpAPI（多引擎代理）

- **API**: `GET https://serpapi.com/search`
- **认证**: Query `api_key={api_key}`
- **参数**: `q`, `engine` (google/baidu/bing), `num`, `hl`
- **返回**: JSON `{ organic_results: [{ title, link, snippet }] }`
- **api_format**: `search_serpapi`
- **特点**: 支持 Google、百度、Bing 多引擎

## 3. 配置存储

复用 `model_configs` 表，通过 `api_format` 前缀 `search_` 区分：

```sql
-- Brave Search 配置
INSERT INTO model_configs (name, api_format, base_url, model_name, api_key, is_default)
VALUES ('Brave Search', 'search_brave', 'https://api.search.brave.com', '', 'BSAxxxxx', 1);

-- 秘塔搜索配置
INSERT INTO model_configs (name, api_format, base_url, model_name, api_key, is_default)
VALUES ('秘塔搜索', 'search_metaso', 'https://api.metaso.cn', '', 'mts-xxxxx', 0);
```

**约定**：
- `api_format` 以 `search_` 前缀 → 搜索 Provider
- `is_default = 1` → 当前激活的搜索 Provider
- `base_url` → 允许用户自定义 API 地址
- `model_name` → 部分 Provider 需要（如 SerpAPI 的 engine）

## 4. WebSearchTool 初始化流程

```
send_message 被调用
  → SELECT * FROM model_configs WHERE api_format LIKE 'search_%' AND is_default=1
  → 若有结果 → 根据 api_format 创建对应 SearchProvider
  → WebSearchTool::with_provider(provider, cache) 注册到 registry
  → 若无结果 → 不注册 web_search 工具（Agent 不调用搜索）
```

## 5. 内存缓存

```rust
pub struct SearchCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    ttl: Duration,      // 15 分钟
    max_size: usize,    // 100 条
}
```

- Key = `"{provider}:{query_lowercase}:{count}"` 规范化
- TTL 过期自动清理
- 超过 max_size 时移除最早条目
- 全局单例，跨会话共享

## 6. 超时机制

| 阶段 | 超时时间 |
|------|---------|
| TCP 连接 | 5 秒 |
| 整体请求 | 15 秒 |

所有 HTTP 请求通过 `reqwest::blocking::Client` + `connect_timeout` + `timeout`。
executor.rs 已用 `spawn_blocking` 包装，不会阻塞 tokio 异步运行时。

## 7. 错误处理

| 场景 | 行为 |
|------|------|
| API Key 无效 (401/403) | 返回 "搜索配置错误: API Key 无效，请在设置中检查" |
| 网络超时 | 返回 "搜索超时: 请检查网络连接" |
| API 限流 (429) | 返回 "搜索频率超限: 请稍后重试" |
| Provider 宕机 (5xx) | 返回 "搜索服务暂不可用: {provider_name}" |
| 无搜索配置 | 不注册 web_search 工具，Agent 无法调用 |

## 8. 安全

- API Key 不出现在工具输出中
- 搜索结果添加外部内容标记（`[搜索结果 - 来自 {provider}]`）
- 结果截断到 30,000 字符

## 9. 工具 Schema

```json
{
  "name": "web_search",
  "description": "搜索互联网获取最新信息。返回网页标题、URL 和摘要。",
  "input_schema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "搜索关键词" },
      "count": { "type": "integer", "description": "返回结果数量 (1-10，默认 5)", "default": 5 },
      "freshness": { "type": "string", "description": "结果新鲜度: day/week/month/year", "enum": ["day", "week", "month", "year"] }
    },
    "required": ["query"]
  }
}
```

## 10. 前端设置页面

在设置页面增加"搜索引擎"配置区域：
- Provider 类型下拉选择（Brave/Tavily/秘塔/博查/SerpAPI）
- API Key 输入框
- 可选：自定义 API 地址
- 测试连接按钮
- 设为默认

## 11. 文件结构

```
src/agent/tools/
├── web_search.rs           # WebSearchTool + SearchProvider trait
├── search_providers/       # 新目录
│   ├── mod.rs             # Provider trait 定义 + 工厂函数
│   ├── brave.rs           # Brave Search
│   ├── tavily.rs          # Tavily
│   ├── metaso.rs          # 秘塔搜索
│   ├── bocha.rs           # 博查搜索
│   └── serpapi.rs         # SerpAPI
└── ...
```
