# 秘塔搜索 API 修复记录

**日期**: 2026-02-24
**问题**: 秘塔搜索测试连接失败
**根本原因**: API 端点和请求参数格式不符合官方规范

## 问题分析

### 1. API 端点错误

**官方正确端点**:
```
POST https://metaso.cn/api/v1/search
```

**代码原实现**:
```rust
// 错误的端点
let url = format!("{}/api/search", self.base_url);
// 配合前端预设 base_url: "https://api.metaso.cn"
// 实际请求: https://api.metaso.cn/api/search ❌
```

### 2. 请求参数格式不匹配

**官方要求** (根据官方示例):
```json
{
  "q": "搜索关键词",
  "scope": "webpage",
  "includeSummary": false,
  "size": "10",
  "includeRawContent": false,
  "conciseSnippet": false
}
```

**代码原实现**:
```json
{
  "query": "搜索关键词",
  "mode": "concise"
}
```

## 修复方案

### 1. 修复 API 端点

**文件**: `apps/runtime/src-tauri/src/agent/tools/search_providers/metaso.rs:66`

```rust
// 使用官方 API 端点：/api/v1/search
let url = format!("{}/api/v1/search", self.base_url);
```

### 2. 修复请求参数

**文件**: `apps/runtime/src-tauri/src/agent/tools/search_providers/metaso.rs:55-63`

```rust
// 使用官方 API 参数格式
let body = json!({
    "q": params.query,
    "scope": "webpage",
    "includeSummary": false,
    "size": params.count.to_string(),
    "includeRawContent": false,
    "conciseSnippet": false
});
```

### 3. 修复前端预设配置

**文件**: `apps/runtime/src/components/SettingsView.tsx:32`

```typescript
// 修复前：base_url: "https://api.metaso.cn" ❌
// 修复后：
{
  label: "秘塔搜索 (中文首选)",
  value: "metaso",
  api_format: "search_metaso",
  base_url: "https://metaso.cn",  // ✅ 官方正确域名
  model_name: ""
}
```

## 官方示例参考

```bash
curl --location 'https://metaso.cn/api/v1/search' \
  --header 'Authorization: Bearer mk-287D151B1DFF5F49ED9E8226E3D0329C' \
  --header 'Accept: application/json' \
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

## 验证结果

### 单元测试通过
```bash
$ cd apps/runtime/src-tauri && cargo test metaso
running 9 tests
test agent::tools::search_providers::metaso::tests::... ok (全部通过)
```

### 使用说明

1. **重启应用**:
   ```bash
   pnpm runtime
   ```

2. **配置秘塔搜索**:
   - 打开设置 → 搜索引擎标签页
   - 快速选择 → "秘塔搜索 (中文首选)"
   - 输入你的秘塔 API 密钥
   - Base URL 会自动填充为 `https://metaso.cn`
   - 点击"测试连接"

3. **如果已有旧配置**: 删除旧的秘塔搜索配置，重新添加

## 影响范围

- ✅ 单元测试全部通过
- ✅ 不影响其他 Provider（Brave, Tavily, Bocha, SerpAPI）
- ✅ 响应解析逻辑保持兼容（支持 `data.items` 和 `results` 两种格式）

## 相关文件

- `apps/runtime/src-tauri/src/agent/tools/search_providers/metaso.rs`
- `apps/runtime/src/components/SettingsView.tsx`
- `docs/plans/2026-02-23-web-search-providers-design.md`
