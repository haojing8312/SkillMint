# Model Provider Catalog Unification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 统一首次模型向导与设置页的服务商目录，补齐官方 API Key 入口，并支持自定义 OpenAI / Claude 两类接入。

**Architecture:** 新增一个前端共享的服务商目录模块，集中定义服务商元数据、推荐模型、官方链接和自定义协议说明。`App.tsx` 与 `SettingsView.tsx` 都改为消费同一份目录和同一套 helper，后端存储与命令保持不变。

**Tech Stack:** React 18, TypeScript, Vitest, Testing Library, Tauri invoke API mocks.

---

### Task 1: 提取统一服务商目录与映射 helper

**Files:**
- Create: `apps/runtime/src/model-provider-catalog.ts`
- Create: `apps/runtime/src/__tests__/model-provider-catalog.test.ts`

**Step 1: Write the failing test**

在 `apps/runtime/src/__tests__/model-provider-catalog.test.ts` 新增最小测试，覆盖：

- 目录包含所有官方服务商和两个自定义项
- 通过目录项生成默认表单值
- 通过 `api_format + base_url` 可回推到官方项
- 未匹配的 `openai` / `anthropic` 配置分别回退到两个自定义项

```typescript
import {
  MODEL_PROVIDER_CATALOG,
  buildModelFormFromCatalogItem,
  resolveCatalogItemForConfig,
} from "../model-provider-catalog";

test("falls back unknown openai config to custom openai", () => {
  const item = resolveCatalogItemForConfig({
    api_format: "openai",
    base_url: "https://proxy.example.com/v1",
  });

  expect(item.id).toBe("custom-openai");
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/model-provider-catalog.test.ts`  
Expected: FAIL with module or symbol not found.

**Step 3: Write minimal implementation**

在 `apps/runtime/src/model-provider-catalog.ts` 实现：

- `MODEL_PROVIDER_CATALOG`
- `DEFAULT_MODEL_PROVIDER_ID`
- `buildModelFormFromCatalogItem`
- `resolveCatalogItemForConfig`
- 必要的类型定义

目录中写入：

- 智谱 GLM
- OpenAI
- Claude (Anthropic)
- MiniMax (OpenAI 兼容)
- MiniMax (Claude 兼容)
- DeepSeek
- Qwen（国际）
- Qwen（国内）
- Moonshot / Kimi
- Yi
- 自定义 OpenAI 兼容
- 自定义 Claude (Anthropic)

并为官方服务商加上 `officialConsoleUrl`，为自定义项加上 `isCustom` 与说明文案。

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/model-provider-catalog.test.ts`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/model-provider-catalog.ts apps/runtime/src/__tests__/model-provider-catalog.test.ts
git commit -m "feat(models): add shared provider catalog and resolution helpers"
```

### Task 2: 先用 TDD 改首次快速向导，改成消费统一目录

**Files:**
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`
- Modify: `apps/runtime/src/App.tsx`

**Step 1: Write the failing test**

在 `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx` 增加测试，覆盖：

- 快速向导中的服务商下拉/卡片展示完整清单而不是 4 项
- 选择 `自定义 Claude (Anthropic)` 后，`test_connection_cmd` 与 `save_model_config` 使用 `api_format: "anthropic"`
- 选择官方服务商时出现“获取 API Key”按钮
- 选择自定义项时不出现官方购买按钮，而出现自定义说明

```typescript
test("switches quick setup to custom anthropic and saves anthropic config", async () => {
  render(<App />);
  // 打开 quick setup，切换到自定义 Claude，填写 API Key 并保存
  // 断言 invoke("save_model_config") 的 config.api_format === "anthropic"
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.model-setup-hint.test.tsx`  
Expected: FAIL because the quick setup still uses the old 4-item preset list and has no official/custom info panel.

**Step 3: Write minimal implementation**

在 `apps/runtime/src/App.tsx`：

- 删除本地 `QUICK_MODEL_PRESETS` 静态定义，改为从 `model-provider-catalog.ts` 读取
- 将 quick setup 初始表单改为由 `DEFAULT_MODEL_PROVIDER_ID` 生成
- 切换服务商时使用统一 helper 更新 `name` / `api_format` / `base_url` / `model_name`
- 在右侧新增服务商说明区：
  - 官方项显示 `获取 API Key` / 可选 `查看文档`
  - 自定义项显示协议说明，不显示官方购买按钮
- 保留现有保存 / 测试命令不变

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.model-setup-hint.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.model-setup-hint.test.tsx
git commit -m "feat(app): unify quick model setup with shared provider catalog"
```

### Task 3: 用 TDD 改设置页，改成消费统一目录

**Files:**
- Create: `apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx`
- Modify: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: Write the failing test**

新增 `apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx`，覆盖：

- 设置页展示完整服务商清单
- 选中官方服务商后，右侧显示官方入口按钮和服务商说明
- 选中自定义 OpenAI 后，保留可编辑 `base_url` / `model_name`
- 编辑已有未知 `openai` 地址时，自动归类到 `自定义 OpenAI 兼容`
- 编辑已有未知 `anthropic` 地址时，自动归类到 `自定义 Claude (Anthropic)`

```typescript
test("maps unknown anthropic config to custom anthropic when editing", async () => {
  // mock list_model_configs 返回一条 anthropic + 非官方 base_url 的配置
  // 点击编辑后断言服务商选择器落在“自定义 Claude (Anthropic)”
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.model-providers.test.tsx`  
Expected: FAIL because `SettingsView` still uses local `PROVIDER_PRESETS` and has no catalog-backed resolution logic.

**Step 3: Write minimal implementation**

在 `apps/runtime/src/components/SettingsView.tsx`：

- 删除本地 `PROVIDER_PRESETS` 静态定义
- 改为使用 `MODEL_PROVIDER_CATALOG`
- 将新增/重置表单默认值切换到统一 helper
- 编辑已有模型时，使用 `resolveCatalogItemForConfig`
- 在模型配置区域增加右侧服务商说明与官方入口按钮
- 官方项仍允许用户手改 `base_url` / `model_name`
- 自定义项保持 `base_url` / `model_name` 可编辑并增加提示文案

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.model-providers.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx
git commit -m "feat(settings): share provider catalog with model configuration view"
```

### Task 4: 补齐外链打开方式和回归测试

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`
- Modify: `apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx`

**Step 1: Write the failing test**

补充测试断言：

- 点击官方入口按钮会触发统一的外链打开逻辑
- 当服务商没有 `officialConsoleUrl` 时，按钮不渲染

测试里直接 mock 统一的打开函数或 `window.open` 调用，而不是分别在两个页面写不同断言。

```typescript
test("does not render official console button for custom providers", async () => {
  // 切换到自定义服务商
  // 断言 screen.queryByRole("link", { name: "获取 API Key" }) 为 null
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.model-setup-hint.test.tsx src/components/__tests__/SettingsView.model-providers.test.tsx`  
Expected: FAIL if the official/custom button visibility or link behavior is still inconsistent.

**Step 3: Write minimal implementation**

收敛外链行为：

- 在 `App.tsx` 和 `SettingsView.tsx` 使用同一套链接字段
- 按目录项是否存在 `officialConsoleUrl` / `officialDocsUrl` 决定按钮显示
- 确保 link/button 的文案和可访问名称一致，便于测试

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.model-setup-hint.test.tsx src/components/__tests__/SettingsView.model-providers.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/components/SettingsView.tsx apps/runtime/src/__tests__/App.model-setup-hint.test.tsx apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx
git commit -m "test(models): cover provider links and custom protocol guidance"
```

### Task 5: 运行完整相关验证并更新说明文档

**Files:**
- Modify: `README.md`
- Modify: `README.en.md`
- Modify: `docs/plans/2026-03-06-model-provider-catalog-unification-design.md`

**Step 1: Write the failing test**

此任务以文档和回归验证为主，不新增功能测试；先记录需要补充的用户说明：

- 首次向导与设置页使用同一清单
- 自定义接入只分 OpenAI / Anthropic 两种协议
- 官方服务商可从右侧跳转申请 API Key

**Step 2: Run focused verification**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/model-provider-catalog.test.ts src/__tests__/App.model-setup-hint.test.tsx src/components/__tests__/SettingsView.model-providers.test.tsx`  
Expected: PASS.

**Step 3: Run broader frontend verification**

Run: `pnpm --dir apps/runtime exec vitest run`  
Expected: PASS with no regressions in existing `App` / `SettingsView` tests.

**Step 4: Write minimal documentation updates**

在 `README.md` 与 `README.en.md` 中补一小节，说明：

- 官方服务商列表
- 自定义 OpenAI / Claude 兼容接入
- 首次向导可直接跳转官方控制台申请 Key

**Step 5: Commit**

```bash
git add README.md README.en.md docs/plans/2026-03-06-model-provider-catalog-unification-design.md docs/plans/2026-03-06-model-provider-catalog-unification-plan.md
git commit -m "docs(models): document unified provider catalog and custom gateway setup"
```
