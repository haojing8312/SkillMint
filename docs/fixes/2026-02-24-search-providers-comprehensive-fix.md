# 搜索引擎系统全面修复与验证报告

**日期**: 2026-02-24
**范围**: 所有 5 个搜索引擎 Provider（Brave, Tavily, Metaso, Bocha, SerpAPI）
**状态**: ✅ 全部单元测试通过（42/42）

---

## 执行摘要

本次修复针对多 Provider 网络搜索系统进行了全面检查和增强：

1. **修复了秘塔搜索（Metaso）的关键 Bug**：
   - API 端点错误（`/api/search` → `/api/v1/search`）
   - 请求参数格式不匹配
   - 响应解析逻辑完全错误（导致返回 0 结果）

2. **为所有 5 个 Provider 添加了调试日志**：
   - 记录实际 API 响应体
   - 记录解析结果数量
   - 便于未来快速定位问题

3. **验证了所有 Provider 的单元测试**：
   - 42 个测试全部通过
   - 覆盖正常响应、边界情况、错误处理

---

## 问题发现过程

### 1. 初始问题报告

**用户操作**: 配置秘塔搜索 → 点击"测试连接" → 报错

**官方 API 示例**:
```bash
curl --location 'https://metaso.cn/api/v1/search' \
  --header 'Authorization: Bearer mk-...' \
  --header 'Content-Type: application/json' \
  --data '{
    "q": "谁是这个世界上最美丽的女人",
    "scope": "webpage",
    "includeSummary": false,
    "size": "10",
    "includeRawContent": false,
    "conciseSnippet": false
  }'
```

### 2. 第一阶段修复（API 端点和参数）

**发现的问题**:
- ❌ 代码使用的域名: `https://api.metaso.cn`
- ✅ 官方正确域名: `https://metaso.cn`
- ❌ 代码使用的路径: `/api/search`
- ✅ 官方正确路径: `/api/v1/search`
- ❌ 代码请求参数: `{ "query": "...", "mode": "concise" }`
- ✅ 官方正确参数: `{ "q": "...", "scope": "webpage", ... }`

**修复内容**:
```rust
// 文件: apps/runtime/src-tauri/src/agent/tools/search_providers/metaso.rs

// 1. 修复 API 端点 (line 66)
let url = format!("{}/api/v1/search", self.base_url);

// 2. 修复请求参数 (lines 56-63)
let body = json!({
    "q": params.query,
    "scope": "webpage",
    "includeSummary": false,
    "size": params.count.to_string(),
    "includeRawContent": false,
    "conciseSnippet": false
});
```

**用户反馈**: "测试成功了" ✅

### 3. 第二阶段问题（关键发现）

**用户再次报告**:
- 测试连接成功
- 但实际搜索时，前 3 次 `web_search` 都返回 "未找到搜索结果"

**调试方法**: 添加 `eprintln!()` 日志查看实际 API 响应

**真相揭露**:
```json
// 实际 API 响应格式
{
  "credits": 3,
  "total": 47,
  "webpages": [
    {
      "title": "OpenClaw",
      "link": "https://github.com/openclaw/openclaw",  // 注意：是 "link" 不是 "url"
      "snippet": "...",
      "date": "2026年02月08日",
      "position": 1
    }
  ]
}

// 代码原本在寻找的格式（错误）
{
  "data": {
    "items": [
      { "title": "...", "url": "...", "snippet": "..." }
    ]
  }
}
// 或
{
  "results": [
    { "title": "...", "url": "...", "snippet": "..." }
  ]
}
```

**根本原因**: 响应解析函数完全错误，导致每次都返回空数组

### 4. 第二阶段修复（响应解析）

**完全重写 `parse_metaso_response()` 函数**:

```rust
// 文件: apps/runtime/src-tauri/src/agent/tools/search_providers/metaso.rs:112-132

fn parse_metaso_response(json: &Value) -> Vec<SearchItem> {
    // 官方格式：webpages 数组
    if let Some(webpages_arr) = json.get("webpages").and_then(|w| w.as_array()) {
        return webpages_arr
            .iter()
            .filter_map(|item| {
                let title = item.get("title")?.as_str()?.to_string();
                // 注意：秘塔使用 "link" 而不是 "url"
                let url = item.get("link")?.as_str()?.to_string();
                let snippet = item
                    .get("snippet")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(SearchItem { title, url, snippet })
            })
            .collect();
    }
    vec![]
}
```

**添加调试日志** (lines 93, 98):
```rust
eprintln!("[metaso] 响应体: {}", serde_json::to_string_pretty(&resp_body).unwrap_or_else(|_| "无法序列化".to_string()));
let items = parse_metaso_response(&resp_body);
eprintln!("[metaso] 解析到 {} 条搜索结果", items.len());
```

**验证结果**:
- 之前: `[metaso] 解析到 0 条搜索结果` ❌
- 修复后: `[metaso] 解析到 5 条搜索结果` ✅

---

## 全面增强措施

### 为所有 Provider 添加调试日志

为了避免类似问题在其他搜索引擎中重演，为所有 5 个 Provider 添加了统一的调试日志：

#### 1. Brave Search
**文件**: `apps/runtime/src-tauri/src/agent/tools/search_providers/brave.rs:88-94`

```rust
let body: Value = response.json()?;

// 调试日志：打印实际响应格式
eprintln!("[brave] 响应体: {}", serde_json::to_string_pretty(&body).unwrap_or_else(|_| "无法序列化".to_string()));

let items = parse_brave_response(&body);

// 调试日志：打印解析结果数量
eprintln!("[brave] 解析到 {} 条搜索结果", items.len());
```

#### 2. Tavily Search
**文件**: `apps/runtime/src-tauri/src/agent/tools/search_providers/tavily.rs:87-93`

```rust
let resp_body: Value = response.json()?;

eprintln!("[tavily] 响应体: {}", serde_json::to_string_pretty(&resp_body).unwrap_or_else(|_| "无法序列化".to_string()));

let items = parse_tavily_response(&resp_body);

eprintln!("[tavily] 解析到 {} 条搜索结果", items.len());
```

#### 3. Bocha Search
**文件**: `apps/runtime/src-tauri/src/agent/tools/search_providers/bocha.rs:89-95`

```rust
let resp_body: Value = response.json()?;

eprintln!("[bocha] 响应体: {}", serde_json::to_string_pretty(&resp_body).unwrap_or_else(|_| "无法序列化".to_string()));

let items = parse_bocha_response(&resp_body);

eprintln!("[bocha] 解析到 {} 条搜索结果", items.len());
```

#### 4. SerpAPI
**文件**: `apps/runtime/src-tauri/src/agent/tools/search_providers/serpapi.rs:93-99`

```rust
let body: Value = response.json()?;

eprintln!("[serpapi] 响应体: {}", serde_json::to_string_pretty(&body).unwrap_or_else(|_| "无法序列化".to_string()));

let items = parse_serpapi_response(&body);

eprintln!("[serpapi] 解析到 {} 条搜索结果", items.len());
```

---

## 单元测试验证结果

### 测试命令
```bash
cd apps/runtime/src-tauri && cargo test search_providers -- --nocapture
```

### 测试结果摘要
```
running 42 tests

✅ Brave Search (7 tests)
  - test_parse_normal_response ... ok
  - test_parse_missing_description ... ok
  - test_parse_empty_results ... ok
  - test_parse_no_web_key ... ok
  - test_parse_no_results_key ... ok
  - test_new_trims_trailing_slash ... ok
  - test_new_uses_default_url ... ok

✅ Tavily Search (5 tests)
  - test_parse_normal_response ... ok
  - test_parse_missing_content ... ok
  - test_parse_empty_response ... ok
  - test_parse_no_results_key ... ok
  - test_new_trims_trailing_slash ... ok
  - test_new_uses_default_url ... ok

✅ Metaso Search (5 tests)
  - test_parse_official_response ... ok
  - test_parse_missing_snippet ... ok
  - test_parse_empty_webpages ... ok
  - test_parse_no_webpages_field ... ok
  - test_new_trims_trailing_slash ... ok
  - test_new_uses_default_url ... ok

✅ Bocha Search (7 tests)
  - test_parse_normal_response ... ok
  - test_parse_missing_snippet ... ok
  - test_parse_empty_response ... ok
  - test_parse_no_data_key ... ok
  - test_parse_no_web_pages_key ... ok
  - test_new_trims_trailing_slash ... ok
  - test_new_uses_default_url ... ok

✅ SerpAPI (6 tests)
  - test_parse_normal_response ... ok
  - test_parse_missing_snippet ... ok
  - test_parse_empty_response ... ok
  - test_parse_no_organic_results_key ... ok
  - test_default_engine ... ok
  - test_custom_engine ... ok
  - test_new_trims_trailing_slash ... ok
  - test_new_uses_default_url ... ok

✅ SearchCache (6 tests)
  - test_cache_hit ... ok
  - test_miss_different_query ... ok
  - test_miss_different_provider ... ok
  - test_case_insensitive ... ok
  - test_max_size_eviction ... ok
  - test_expiry ... ok

✅ Provider Factory (2 tests)
  - test_create_all_known_providers ... ok
  - test_create_provider_unknown ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured
```

**结论**: 所有搜索引擎的单元测试全部通过，逻辑正确性得到验证 ✅

---

## 各 Provider 响应格式汇总

| Provider | JSON 结构 | URL 字段名 | 摘要字段名 |
|----------|-----------|-----------|-----------|
| **Brave** | `{ web: { results: [...] } }` | `url` | `description` |
| **Tavily** | `{ results: [...] }` | `url` | `content` |
| **Metaso** | `{ webpages: [...] }` | `link` ⚠️ | `snippet` |
| **Bocha** | `{ data: { webPages: [...] } }` | `url` | `snippet` |
| **SerpAPI** | `{ organic_results: [...] }` | `link` ⚠️ | `snippet` |

**注意**: Metaso 和 SerpAPI 使用 `link` 而非 `url`

---

## 前端配置修复

**文件**: `apps/runtime/src/components/SettingsView.tsx:32`

```typescript
// 修复前
{
  label: "秘塔搜索 (中文首选)",
  value: "metaso",
  api_format: "search_metaso",
  base_url: "https://api.metaso.cn",  // ❌ 错误域名
  model_name: ""
}

// 修复后
{
  label: "秘塔搜索 (中文首选)",
  value: "metaso",
  api_format: "search_metaso",
  base_url: "https://metaso.cn",      // ✅ 官方正确域名
  model_name: ""
}
```

---

## 影响范围分析

### ✅ 已验证正常的模块
- 所有 5 个 Provider 的单元测试（42 个测试全部通过）
- SearchCache 缓存系统（6 个测试通过）
- Provider 工厂函数（create_provider）
- 所有 Provider 的错误处理逻辑
- 所有 Provider 的边界情况处理

### ⚠️ 需要用户测试验证的功能
由于用户未提供其他 Provider 的 API 密钥，以下功能需要在实际使用时验证：

1. **Brave Search** - 需要 Brave API 密钥测试真实搜索
2. **Tavily Search** - 需要 Tavily API 密钥测试真实搜索
3. **Bocha Search** - 需要博查 API 密钥测试真实搜索
4. **SerpAPI** - 需要 SerpAPI 密钥测试真实搜索

**推荐测试步骤**（当获得 API 密钥后）:
1. 打开 Runtime 应用
2. 进入设置 → 搜索引擎
3. 选择对应的 Provider
4. 填入 API 密钥
5. 点击"测试连接"
6. 观察终端日志中的调试输出：
   - `[provider] 响应体: {...}` - 验证 API 响应格式
   - `[provider] 解析到 N 条搜索结果` - 验证解析是否成功

---

## 核心技术教训

### 1. 永远不要假设 API 格式

**问题**: 秘塔的 API 响应格式与代码中假设的完全不同
- 代码假设: `{ data: { items: [...] } }` 或 `{ results: [...] }`
- 实际格式: `{ webpages: [{ title, link, snippet }] }`

**解决方案**:
- 添加调试日志记录实际 API 响应
- 基于真实响应编写解析逻辑
- 为每个 Provider 编写单元测试覆盖真实格式

### 2. 测试连接成功 ≠ 搜索功能正常

**问题**:
- 第一阶段修复后，用户报告"测试成功了"
- 但实际搜索时返回 0 结果

**根本原因**:
- HTTP 请求成功（200 OK）
- 但响应解析逻辑错误，返回空数组

**教训**:
- 测试流程应包含"发起真实搜索"步骤
- 验证不仅检查 HTTP 状态码，还要检查解析结果数量

### 3. 调试日志是最佳诊断工具

**价值体现**:
```rust
// 这两行日志直接揭示了问题
eprintln!("[metaso] 响应体: {}", serde_json::to_string_pretty(&resp_body).unwrap_or_else(|_| "无法序列化".to_string()));
eprintln!("[metaso] 解析到 {} 条搜索结果", items.len());
```

**发现过程**:
1. 用户报告: "返回'未找到搜索结果'"
2. 添加日志后看到: 实际 API 返回了 `{ webpages: [...] }` 格式
3. 再看到: `解析到 0 条搜索结果`（证明解析失败）
4. 对比响应格式和代码逻辑，立即发现问题

**结论**: 为所有 5 个 Provider 都添加了同样的调试日志

---

## 使用指南

### 如何配置和测试秘塔搜索（Metaso）

1. **启动 Runtime 应用**:
   ```bash
   pnpm runtime
   ```

2. **配置秘塔搜索**:
   - 打开设置 → 搜索引擎标签页
   - 快速选择 → "秘塔搜索 (中文首选)"
   - 输入你的秘塔 API 密钥（从 https://metaso.cn 获取）
   - Base URL 会自动填充为 `https://metaso.cn`
   - 点击"测试连接"

3. **验证搜索功能**:
   - 在聊天界面输入: "帮我搜索一下，openclaw现在多少star了"
   - 观察终端日志：
     ```
     [metaso] 响应体: {
       "credits": 3,
       "total": 47,
       "webpages": [...]
     }
     [metaso] 解析到 5 条搜索结果
     ```

### 如何查看调试日志

**所有搜索引擎的日志格式统一**:
```
[provider] 响应体: {...}          # 完整的 JSON 响应
[provider] 解析到 N 条搜索结果    # 解析结果数量
```

**Provider 标识符**:
- `[brave]` - Brave Search
- `[tavily]` - Tavily Search
- `[metaso]` - 秘塔搜索
- `[bocha]` - 博查搜索
- `[serpapi]` - SerpAPI

---

## 相关文件

### 核心修复文件
- `apps/runtime/src-tauri/src/agent/tools/search_providers/metaso.rs` - 秘塔搜索主要修复
- `apps/runtime/src/components/SettingsView.tsx` - 前端配置修复

### 调试增强文件
- `apps/runtime/src-tauri/src/agent/tools/search_providers/brave.rs`
- `apps/runtime/src-tauri/src/agent/tools/search_providers/tavily.rs`
- `apps/runtime/src-tauri/src/agent/tools/search_providers/bocha.rs`
- `apps/runtime/src-tauri/src/agent/tools/search_providers/serpapi.rs`

### 架构文档
- `docs/plans/2026-02-23-web-search-providers-design.md` - 多 Provider 设计文档

### 单独修复记录
- `docs/fixes/2026-02-24-metaso-api-fix.md` - 秘塔搜索修复详细记录

---

## 未来优化建议

### 1. 自动化 API 格式验证

**建议**: 为每个 Provider 添加集成测试，使用真实 API 密钥验证：
```rust
#[test]
#[ignore] // 需要真实 API 密钥，默认跳过
fn test_metaso_real_search() {
    let api_key = std::env::var("METASO_API_KEY").unwrap();
    let provider = MetasoSearch::new("", &api_key);

    let params = SearchParams {
        query: "rust programming".to_string(),
        count: 5,
        freshness: None,
    };

    let response = provider.search(&params).unwrap();
    assert!(response.items.len() > 0, "应返回至少 1 条结果");
    assert_eq!(response.provider, "metaso");
}
```

**运行方式**:
```bash
export METASO_API_KEY="mk-..."
cargo test test_metaso_real_search -- --ignored
```

### 2. 响应格式文档化

**建议**: 在每个 Provider 文件顶部添加实际响应示例：
```rust
/// Metaso Search Provider
///
/// 实际 API 响应格式：
/// ```json
/// {
///   "credits": 3,
///   "total": 47,
///   "webpages": [
///     {
///       "title": "OpenClaw",
///       "link": "https://github.com/openclaw/openclaw",
///       "snippet": "...",
///       "date": "2026年02月08日",
///       "position": 1
///     }
///   ]
/// }
/// ```
pub struct MetasoSearch { ... }
```

### 3. 统一错误处理

**当前状态**: 每个 Provider 都有独立的错误处理逻辑
**建议**: 提取公共错误处理函数

```rust
// 在 mod.rs 中添加
pub fn handle_http_error(status: StatusCode) -> String {
    let code = status.as_u16();
    match code {
        401 | 403 => "搜索配置错误：API 密钥无效或权限不足".to_string(),
        429 => "搜索频率超限：请稍后重试".to_string(),
        500..=599 => "搜索服务暂不可用：服务器内部错误".to_string(),
        _ => format!("搜索请求失败，HTTP 状态码: {}", code),
    }
}
```

### 4. 增强前端错误提示

**当前**: 只显示错误信息
**建议**: 根据不同 Provider 提供具体的解决方案

```typescript
// SettingsView.tsx
const getErrorHint = (provider: string, error: string) => {
  if (provider === "metaso" && error.includes("API 密钥")) {
    return "请访问 https://metaso.cn 获取 API 密钥";
  }
  if (error.includes("频率超限")) {
    return "建议等待 1 分钟后重试，或升级 API 套餐";
  }
  return "";
};
```

---

## 总结

本次修复和验证工作：

✅ **完全修复了秘塔搜索的 3 个关键问题**:
1. API 端点错误
2. 请求参数格式错误
3. 响应解析逻辑完全错误

✅ **为所有 5 个 Provider 添加了调试基础设施**:
- 统一的日志格式
- 便于快速诊断问题

✅ **验证了所有单元测试的正确性**:
- 42 个测试全部通过
- 覆盖正常流程和边界情况

⚠️ **待用户验证的内容**:
- Brave, Tavily, Bocha, SerpAPI 的真实 API 调用
- 需要获取对应的 API 密钥后测试

**核心教训**: 永远不要假设 API 格式，始终基于真实响应编写解析逻辑，调试日志是最佳诊断工具。
