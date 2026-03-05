# 默认语言与沉浸式翻译 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 WorkClaw 中落地“默认语言 + 默认沉浸式翻译”，并在技能库、找技能、聊天安装候选卡片中自动翻译英文内容（优先使用用户已配置默认大模型）。

**Architecture:** 以后端统一翻译服务为中心，运行时偏好定义语言与开关，前端通过通用 Hook 获取翻译映射并统一渲染。翻译仅作用展示层，不改变安装/检索所依赖的 slug、id、原始参数。

**Tech Stack:** Rust (Tauri commands/sqlx/reqwest), TypeScript + React + Vitest, SQLite `app_settings` + `skill_i18n_cache`.

---

### Task 1: 扩展运行时偏好模型（默认语言 + 沉浸式翻译开关）

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/runtime_preferences.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Modify: `apps/runtime/src/types.ts`
- Test: `apps/runtime/src-tauri/tests/test_runtime_preferences.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn runtime_preferences_have_language_and_immersive_defaults() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let prefs = get_runtime_preferences_with_pool(&pool).await.unwrap();
    assert_eq!(prefs.default_language, "zh-CN");
    assert!(prefs.immersive_translation_enabled);
    assert_eq!(prefs.immersive_translation_display, "translated_only");
}
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_runtime_preferences runtime_preferences_have_language_and_immersive_defaults -- --nocapture`  
Expected: FAIL with missing struct fields / assertion mismatch.

**Step 3: Write minimal implementation**

```rust
pub struct RuntimePreferences {
    pub default_work_dir: String,
    pub default_language: String,
    pub immersive_translation_enabled: bool,
    pub immersive_translation_display: String,
}
```

同时在 `db.rs` 初始化默认 key：

```sql
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('runtime_default_language', 'zh-CN');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('runtime_immersive_translation_enabled', 'true');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('runtime_immersive_translation_display', 'translated_only');
```

`RuntimePreferencesInput` 使用可选字段并做 merge，避免影响旧调用方（如 EmployeeHub 仅提交 `default_work_dir`）。

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_runtime_preferences -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/runtime_preferences.rs apps/runtime/src-tauri/src/db.rs apps/runtime/src/types.ts apps/runtime/src-tauri/tests/test_runtime_preferences.rs
git commit -m "feat: add language and immersive translation runtime preferences"
```

### Task 2: 实现后端通用翻译服务（优先默认模型）

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/clawhub.rs`
- Modify: `apps/runtime/src-tauri/src/commands/models.rs` (仅复用查询函数时需要)
- Test: `apps/runtime/src-tauri/tests/test_clawhub_translation_preferences.rs` (new)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn translate_respects_disabled_flag_and_returns_source() {
    // set runtime_immersive_translation_enabled=false
    // call translate_texts_with_preferences(["Video Maker"])
    // expect ["Video Maker"]
}
```

补一个缓存键测试（语言维度隔离）：

```rust
#[tokio::test]
async fn translate_cache_key_includes_target_language() { /* ... */ }
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_clawhub_translation_preferences -- --nocapture`  
Expected: FAIL because command/helper not found.

**Step 3: Write minimal implementation**

核心新增（示意）：

```rust
async fn translate_text_with_model_or_fallback(
    pool: &SqlitePool,
    target_lang: &str,
    text: &str,
) -> Result<String, String> { /* 1) load default model 2) adapters::openai/anthropic 3) fallback */ }
```

实现策略：
- 优先读取默认模型配置（`model_configs.is_default=1`）并用 `adapters::openai`/`adapters::anthropic` 进行翻译。
- 若无可用默认模型或请求失败，回退到现有轻量翻译逻辑（或直接原文回退，按代码可控性二选一）。
- 缓存 key 从 `zh-CN:<hash>` 升级为 `<target_lang>:<engine>:<hash>`。

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_clawhub_translation_preferences -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/clawhub.rs apps/runtime/src-tauri/tests/test_clawhub_translation_preferences.rs
git commit -m "feat: add model-driven immersive translation backend service"
```

### Task 3: 新增通用 Tauri 翻译命令并保持兼容

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/clawhub.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/tests/test_clawhub_translation_preferences.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn translate_texts_with_preferences_returns_stable_order() {
    // input: ["A", "B", "A"]
    // output len == 3 and same index mapping
}
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_clawhub_translation_preferences translate_texts_with_preferences_returns_stable_order -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

新增命令：

```rust
#[tauri::command]
pub async fn translate_texts_with_preferences(
    texts: Vec<String>,
    scene: Option<String>,
    db: State<'_, DbState>,
) -> Result<Vec<String>, String> { /* ... */ }
```

兼容：
- `translate_clawhub_texts` 保留，内部调用 `translate_texts_with_preferences(texts, Some("clawhub"))`。
- `lib.rs` 注册新命令，旧命令继续可用。

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_clawhub_translation_preferences -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/clawhub.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_clawhub_translation_preferences.rs
git commit -m "refactor: expose generic translation command with compatibility wrapper"
```

### Task 4: 前端抽取通用沉浸式翻译 Hook

**Files:**
- Create: `apps/runtime/src/hooks/useImmersiveTranslation.ts`
- Modify: `apps/runtime/src/types.ts`
- Test: `apps/runtime/src/components/experts/__tests__/SkillLibraryView.translation.test.tsx` (new)

**Step 1: Write the failing test**

```tsx
test("translates visible texts and falls back on error", async () => {
  // mock invoke("translate_texts_with_preferences")
  // expect translated text rendered
  // then reject and expect source fallback
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/components/experts/__tests__/SkillLibraryView.translation.test.tsx`  
Expected: FAIL (hook/command missing).

**Step 3: Write minimal implementation**

```ts
export function useImmersiveTranslation(texts: string[]) {
  // dedupe -> invoke translate_texts_with_preferences -> map by original text
}
```

约束：
- 仅维护展示 `translatedMap`；
- 不改入参对象的业务字段；
- 返回 `isTranslating` 供页面选择 loading UI。

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/components/experts/__tests__/SkillLibraryView.translation.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/hooks/useImmersiveTranslation.ts apps/runtime/src/components/experts/__tests__/SkillLibraryView.translation.test.tsx apps/runtime/src/types.ts
git commit -m "feat: add reusable immersive translation hook for frontend"
```

### Task 5: 接入技能库与找技能页面

**Files:**
- Modify: `apps/runtime/src/components/experts/SkillLibraryView.tsx`
- Modify: `apps/runtime/src/components/experts/FindSkillsView.tsx`
- Create: `apps/runtime/src/components/experts/__tests__/FindSkillsView.translation.test.tsx`
- Test: `apps/runtime/src/components/experts/__tests__/SkillLibraryView.translation.test.tsx`

**Step 1: Write the failing test**

```tsx
test("find-skills cards render translated name/description/reason", async () => {
  // mock recommendations in English
  // mock translated map in Chinese
  // expect translated content appears
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/components/experts/__tests__/FindSkillsView.translation.test.tsx`  
Expected: FAIL.

**Step 3: Write minimal implementation**

在两个页面统一通过 `useImmersiveTranslation` 渲染：

```tsx
{translatedMap[item.name] ?? item.name}
{translatedMap[item.description] ?? item.description}
```

安装确认弹窗标题也使用译文主显示，原文可作为 `impact` 或小字补充（若启用双语模式）。

**Step 4: Run test to verify it passes**

Run:  
`pnpm --dir apps/runtime test -- src/components/experts/__tests__/SkillLibraryView.translation.test.tsx`  
`pnpm --dir apps/runtime test -- src/components/experts/__tests__/FindSkillsView.translation.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/experts/SkillLibraryView.tsx apps/runtime/src/components/experts/FindSkillsView.tsx apps/runtime/src/components/experts/__tests__/SkillLibraryView.translation.test.tsx apps/runtime/src/components/experts/__tests__/FindSkillsView.translation.test.tsx
git commit -m "feat: apply immersive translation in experts library and finder views"
```

### Task 6: 接入 ChatView 的“可安装技能”卡片

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx`

**Step 1: Write the failing test**

```tsx
test("chat install candidates show translated text but keep raw install args", async () => {
  // output has English candidate
  // translated map returns Chinese
  // click install: still sends slug=video-maker and original githubUrl
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/ChatView.find-skills-install.test.tsx`  
Expected: FAIL.

**Step 3: Write minimal implementation**

对 `pendingInstallSkill.name`、候选卡片 `name/description` 使用译文 map；  
`install_clawhub_skill` 参数继续使用候选原始 `slug/githubUrl/sourceUrl`。

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/ChatView.find-skills-install.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx
git commit -m "feat: add immersive translation for chat install candidates"
```

### Task 7: 在设置中心提供语言与沉浸式翻译配置

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/types.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.translation-preferences.test.tsx` (new)

**Step 1: Write the failing test**

```tsx
test("settings can load and save default language + immersive translation", async () => {
  // mock get_runtime_preferences/set_runtime_preferences
  // change language to zh-CN and toggle immersive translation
  // expect save payload contains new fields
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/SettingsView.translation-preferences.test.tsx`  
Expected: FAIL.

**Step 3: Write minimal implementation**

新增设置区块“语言与沉浸式翻译”：
- 默认语言下拉（首期 `zh-CN`，预留 `en-US`）。
- 沉浸式翻译开关。
- 显示模式选择（`translated_only` / `bilingual_inline`）。

保存时调用：

```ts
invoke("set_runtime_preferences", {
  input: {
    default_work_dir: existing.default_work_dir,
    default_language,
    immersive_translation_enabled,
    immersive_translation_display,
  },
});
```

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/SettingsView.translation-preferences.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.translation-preferences.test.tsx apps/runtime/src/types.ts
git commit -m "feat: add language and immersive translation settings in settings view"
```

### Task 8: 全量验证与文档同步

**Files:**
- Modify: `README.md` (如需补一段功能说明)
- Modify: `README.en.md` (可选同步)
- Modify: `README.zh-CN.md`（若存在独立中文文档，按实际路径更新）

**Step 1: Write the failing test**

无新增失败测试，执行回归验证清单。

**Step 2: Run verification commands**

Run:
- `pnpm --dir apps/runtime test`
- `cd apps/runtime/src-tauri && cargo test --test test_runtime_preferences -- --nocapture`
- `cd apps/runtime/src-tauri && cargo test --test test_clawhub_translation_preferences -- --nocapture`

Expected: all PASS.

**Step 3: Write minimal implementation**

根据结果修复回归点；更新 README 的“设置中心/专家技能中心”说明，明确：
- 默认语言
- 默认沉浸式翻译
- 翻译失败回退原文，不影响安装流程

**Step 4: Re-run verification**

重复 Step 2 所有命令并确认通过。

**Step 5: Commit**

```bash
git add README.md README.en.md README.zh-CN.md
git commit -m "docs: document default language and immersive translation behavior"
```

## Execution Notes

- 执行顺序严格按任务号，使用 @test-driven-development：先写失败测试，再最小实现，再回归。
- 每个任务完成后运行对应最小测试，不等到最后一次性跑全量。
- 提交粒度保持“单一目标”，便于回滚与 code review。
- 收尾前执行 @verification-before-completion，确保结论有命令输出证据。
