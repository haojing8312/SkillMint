# Provider Plugin Model Coverage (China-First) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade SkillHub runtime from single-model config to provider-plugin + capability routing so chat/vision/image/audio can support as many China-first cloud models as possible with fallback.

**Architecture:** Keep protocol adapters (`openai` and `anthropic`) as low-level transport, then add a provider plugin layer and a capability router above it. Provider plugins own auth, health checks, model discovery, and capability metadata. Runtime chooses `provider/model` by capability policy and falls back automatically on failures.

**Tech Stack:** Rust (Tauri, sqlx, anyhow, serde_json), TypeScript/React (SettingsView), SQLite, existing runtime agent executor and adapter modules.

---

### Task 1: Add provider-capability schema and migration path

**Files:**
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Modify: `apps/runtime/src-tauri/src/commands/models.rs`
- Test: `apps/runtime/src-tauri/tests/test_skill_config.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn provider_tables_exist_after_init() {
    // assert provider_configs/provider_capabilities/model_catalog_cache/routing_policies exist
}
```

**Step 2: Run test to verify it fails**

Run: `cmd.exe /c cargo test --test test_skill_config provider_tables_exist_after_init -- --nocapture`  
Expected: FAIL with missing table assertion

**Step 3: Write minimal implementation**

```rust
// db.rs init migration SQL
CREATE TABLE IF NOT EXISTS provider_configs (...);
CREATE TABLE IF NOT EXISTS provider_capabilities (...);
CREATE TABLE IF NOT EXISTS model_catalog_cache (...);
CREATE TABLE IF NOT EXISTS routing_policies (...);
```

**Step 4: Run test to verify it passes**

Run: `cmd.exe /c cargo test --test test_skill_config provider_tables_exist_after_init -- --nocapture`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/src/commands/models.rs apps/runtime/src-tauri/tests/test_skill_config.rs
git commit -m "feat(db): add provider and capability routing tables"
```

### Task 2: Introduce provider trait and registry core

**Files:**
- Create: `apps/runtime/src-tauri/src/providers/mod.rs`
- Create: `apps/runtime/src-tauri/src/providers/traits.rs`
- Create: `apps/runtime/src-tauri/src/providers/registry.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/tests/test_registry.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn registry_can_register_and_lookup_provider() {
    // register fake provider, assert by key and capability
}
```

**Step 2: Run test to verify it fails**

Run: `cmd.exe /c cargo test --test test_registry registry_can_register_and_lookup_provider -- --nocapture`  
Expected: FAIL with unresolved providers module

**Step 3: Write minimal implementation**

```rust
pub trait ProviderPlugin {
    fn key(&self) -> &str;
    fn supports(&self, capability: &str) -> bool;
}
pub struct ProviderRegistry { /* register/get/list */ }
```

**Step 4: Run test to verify it passes**

Run: `cmd.exe /c cargo test --test test_registry registry_can_register_and_lookup_provider -- --nocapture`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/providers apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_registry.rs
git commit -m "feat(runtime): add provider plugin registry core"
```

### Task 3: Build capability router with fallback policy

**Files:**
- Create: `apps/runtime/src-tauri/src/providers/capability_router.rs`
- Modify: `apps/runtime/src-tauri/src/commands/models.rs`
- Test: `apps/runtime/src-tauri/tests/test_react_loop.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn router_uses_fallback_on_primary_error() {
    // primary returns auth/rate-limit, router selects fallback provider/model
}
```

**Step 2: Run test to verify it fails**

Run: `cmd.exe /c cargo test --test test_react_loop router_uses_fallback_on_primary_error -- --nocapture`  
Expected: FAIL with router not found

**Step 3: Write minimal implementation**

```rust
pub struct RoutingPolicy { capability: String, primary: String, fallbacks: Vec<String> }
pub fn route_with_fallback(...) -> Result<ResolvedRoute> { ... }
```

**Step 4: Run test to verify it passes**

Run: `cmd.exe /c cargo test --test test_react_loop router_uses_fallback_on_primary_error -- --nocapture`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/providers/capability_router.rs apps/runtime/src-tauri/src/commands/models.rs apps/runtime/src-tauri/tests/test_react_loop.rs
git commit -m "feat(router): add capability routing and fallback chain"
```

### Task 4: Add China-first P0 provider plugins (chat focus first)

**Files:**
- Create: `apps/runtime/src-tauri/src/providers/openai_compat.rs`
- Create: `apps/runtime/src-tauri/src/providers/anthropic_compat.rs`
- Create: `apps/runtime/src-tauri/src/providers/deepseek.rs`
- Create: `apps/runtime/src-tauri/src/providers/qwen.rs`
- Create: `apps/runtime/src-tauri/src/providers/moonshot.rs`
- Modify: `apps/runtime/src-tauri/src/providers/registry.rs`
- Test: `apps/runtime/src-tauri/tests/test_openai_tools.rs`
- Test: `apps/runtime/src-tauri/tests/test_anthropic_tools.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn deepseek_plugin_declares_chat_capability() {}
#[test]
fn qwen_plugin_declares_vision_capability() {}
#[test]
fn moonshot_plugin_declares_long_context_chat() {}
```

**Step 2: Run tests to verify they fail**

Run: `cmd.exe /c cargo test --test test_openai_tools -- --nocapture`  
Run: `cmd.exe /c cargo test --test test_anthropic_tools -- --nocapture`  
Expected: FAIL with plugin declarations missing

**Step 3: Write minimal implementation**

```rust
impl ProviderPlugin for DeepSeekProvider { ... }
impl ProviderPlugin for QwenProvider { ... }
impl ProviderPlugin for MoonshotProvider { ... }
```

**Step 4: Run tests to verify they pass**

Run: `cmd.exe /c cargo test --test test_openai_tools -- --nocapture`  
Run: `cmd.exe /c cargo test --test test_anthropic_tools -- --nocapture`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/providers apps/runtime/src-tauri/tests/test_openai_tools.rs apps/runtime/src-tauri/tests/test_anthropic_tools.rs
git commit -m "feat(providers): add china-first p0 provider plugins"
```

### Task 5: Wire capability router into agent send_message flow

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/types.rs`
- Test: `apps/runtime/src-tauri/tests/test_e2e_flow.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn send_message_routes_by_chat_capability_not_single_model_id() {
    // create routing policy and assert selected provider/model from policy
}
```

**Step 2: Run test to verify it fails**

Run: `cmd.exe /c cargo test --test test_e2e_flow send_message_routes_by_chat_capability_not_single_model_id -- --nocapture`  
Expected: FAIL with model_id-only path

**Step 3: Write minimal implementation**

```rust
let route = capability_router.resolve("chat")?;
let (provider, model) = route.primary();
executor.execute_turn_with_provider(provider, model, ...);
```

**Step 4: Run test to verify it passes**

Run: `cmd.exe /c cargo test --test test_e2e_flow send_message_routes_by_chat_capability_not_single_model_id -- --nocapture`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/types.rs apps/runtime/src-tauri/tests/test_e2e_flow.rs
git commit -m "feat(agent): route chat by provider capability policy"
```

### Task 6: Extend settings UI for provider, capability, and health tabs

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/components/SettingsView.tsx` (component test if available)

**Step 1: Write the failing UI test**

```tsx
it("renders provider and capability tabs with china-first presets", () => {
  // expect tabs: Providers / Capabilities / Health
});
```

**Step 2: Run test to verify it fails**

Run: `cmd.exe /c pnpm --filter runtime test`  
Expected: FAIL with missing tabs/preset controls

**Step 3: Write minimal implementation**

```tsx
type SettingsTab = "models" | "providers" | "capabilities" | "health" | "mcp" | "search" | "routing";
// add forms for provider auth, capability default+fallback ordering, health status
```

**Step 4: Run test to verify it passes**

Run: `cmd.exe /c pnpm --filter runtime test`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/types.ts apps/runtime/src/App.tsx
git commit -m "feat(ui): add provider capability and health management tabs"
```

### Task 7: Add model discovery, cache, and health-check commands

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/models.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/src/providers/registry.rs`
- Test: `apps/runtime/src-tauri/tests/test_tools_complete.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn list_provider_models_uses_cache_and_ttl() {}
#[test]
fn provider_health_check_reports_latency_and_error() {}
```

**Step 2: Run test to verify it fails**

Run: `cmd.exe /c cargo test --test test_tools_complete -- --nocapture`  
Expected: FAIL with missing commands

**Step 3: Write minimal implementation**

```rust
#[tauri::command] async fn list_provider_models(provider_id: String, capability: Option<String>) -> Result<Vec<ModelInfo>, String> { ... }
#[tauri::command] async fn check_provider_health(provider_id: String) -> Result<HealthInfo, String> { ... }
```

**Step 4: Run test to verify it passes**

Run: `cmd.exe /c cargo test --test test_tools_complete -- --nocapture`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/models.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/providers/registry.rs apps/runtime/src-tauri/tests/test_tools_complete.rs
git commit -m "feat(models): add model discovery cache and provider health commands"
```

### Task 8: Add multimodal capability policies (vision/image/audio) and end-to-end fallback verification

**Files:**
- Modify: `apps/runtime/src-tauri/src/providers/capability_router.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src-tauri/tests/test_e2e_flow.rs`
- Test: `apps/runtime/src-tauri/tests/test_sidecar_bridge.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn image_generation_routes_to_image_policy_chain() {}
#[test]
fn vision_request_uses_vision_policy_chain() {}
#[test]
fn audio_stt_falls_back_when_primary_timeout() {}
```

**Step 2: Run tests to verify they fail**

Run: `cmd.exe /c cargo test --test test_e2e_flow -- --nocapture`  
Run: `cmd.exe /c cargo test --test test_sidecar_bridge -- --nocapture`  
Expected: FAIL with missing capability route branches

**Step 3: Write minimal implementation**

```rust
match capability {
  "chat" => ...,
  "vision" => ...,
  "image_gen" => ...,
  "audio_stt" => ...,
  "audio_tts" => ...,
  _ => ...
}
```

**Step 4: Run tests to verify they pass**

Run: `cmd.exe /c cargo test --test test_e2e_flow -- --nocapture`  
Run: `cmd.exe /c cargo test --test test_sidecar_bridge -- --nocapture`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/providers/capability_router.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src/components/SettingsView.tsx apps/runtime/src-tauri/tests/test_e2e_flow.rs apps/runtime/src-tauri/tests/test_sidecar_bridge.rs
git commit -m "feat(multimodal): add capability policies for vision image and audio with fallback"
```

### Task 9: Documentation and migration notes

**Files:**
- Create: `docs/plans/2026-02-28-provider-plugin-model-coverage-design.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/plans/2026-02-19-llm-adapter-provider-presets-design.md`

**Step 1: Write failing docs checklist**

```md
- [ ] Provider plugin architecture documented
- [ ] Capability routing setup documented
- [ ] China-first provider preset table documented
- [ ] Migration from model_configs-only documented
```

**Step 2: Run docs verification**

Run: `cmd.exe /c rg -n "provider plugin|capability routing|China-first" README.md README.zh-CN.md docs/plans`  
Expected: missing entries

**Step 3: Write minimal implementation**

```md
Add setup examples for DeepSeek/Qwen/Kimi/OpenAI/Anthropic/Gemini.
Add fallback policy examples for chat/vision/image/audio.
```

**Step 4: Run docs verification again**

Run: `cmd.exe /c rg -n "provider plugin|capability routing|China-first" README.md README.zh-CN.md docs/plans`  
Expected: matches in all target docs

**Step 5: Commit**

```bash
git add README.md README.zh-CN.md docs/plans/2026-02-28-provider-plugin-model-coverage-design.md docs/plans/2026-02-19-llm-adapter-provider-presets-design.md
git commit -m "docs: add provider plugin and capability routing migration guide"
```

### Task 10: Final verification before merge

**Files:**
- Modify (if needed): `apps/runtime/src-tauri/tests/*`

**Step 1: Run runtime type/build checks**

Run: `cmd.exe /c pnpm --filter runtime build`  
Expected: build succeeds

**Step 2: Run core Rust integration tests**

Run: `cmd.exe /c cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --tests -- --nocapture`  
Expected: all related tests pass

**Step 3: Run focused smoke checks**

Run: `cmd.exe /c cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_e2e_flow -- --nocapture`  
Run: `cmd.exe /c cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_openai_tools -- --nocapture`  
Run: `cmd.exe /c cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_anthropic_tools -- --nocapture`  
Expected: pass without flaky fallback logic failures

**Step 4: Manual runtime sanity**

Run app and verify:
- provider add/edit/delete
- capability route save/load
- primary failure triggers fallback
- model discovery cache refresh

**Step 5: Commit release readiness**

```bash
git add -A
git commit -m "chore: finalize provider-plugin model coverage rollout checks"
```

