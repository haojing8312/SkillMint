# Search Setup Fallback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let first-use quick setup skip search configuration while preserving a clear runtime fallback path for search-engine, MCP-backed search, or offline responses.

**Architecture:** Keep the existing quick-setup flow and `web_search` tool contract stable. Relax the first-launch search-step gate in the React coordinator, then add a small Rust resolver that prefers the configured search provider, falls back to a compatible MCP search tool by aliasing it to `web_search`, and otherwise leaves the runtime without `web_search`. Search-capable skills remain outside the formal fallback chain for now; they are still available for agent-driven use, but not treated as deterministic runtime fallback.

**Tech Stack:** React, TypeScript, Tauri, Rust, Vitest/Jest-style runtime UI tests, Rust integration/unit tests.

---

### Task 1: First-launch quick setup skip

**Files:**
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`
- Modify: `apps/runtime/src/scenes/useQuickSetupCoordinator.ts`
- Modify: `apps/runtime/src/components/ModelSetupOverlays.tsx`

**Steps:**
1. Add a failing UI test proving first-launch quick setup can skip the search step and still proceed to the optional Feishu step.
2. Run the focused UI test and confirm it fails for the expected reason.
3. Relax the search-step gate so first-launch behaves like the agreed design.
4. Update the quick-setup copy so the first-launch messaging no longer claims search is mandatory.
5. Re-run the focused UI test until it passes.

### Task 2: Runtime search fallback resolution

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`
- Add or modify adjacent Rust tests near runtime tool setup or web search behavior

**Steps:**
1. Add a failing Rust test for selecting a configured search provider first, then an MCP-compatible fallback, then no runtime `web_search`.
2. Run the focused Rust test and confirm the failure is correct.
3. Extract a small resolver/helper for runtime search source selection and alias an MCP tool to `web_search` when needed.
4. Include source-specific output/copy so session traces show where results came from.
5. Re-run the focused Rust tests until they pass.

### Task 3: Verification

**Files:**
- No code changes expected unless verification exposes gaps

**Steps:**
1. Run the smallest honest verification commands for the changed runtime UI and Rust behavior.
2. Record which changed surfaces were covered and what remains unverified.
