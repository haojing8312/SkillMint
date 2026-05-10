# Profile Memory Locator Slice 2 Plan

Date: 2026-05-06

Status: `[~]`

Goal: introduce a read-only profile memory locator that prepares the Memory OS migration without moving files, changing prompt output, or changing `MemoryTool` write behavior.

## Scope

- Add a locator that understands the future target path: `profiles/<profile_id>/memories`.
- Preserve the current legacy bucket shape:
  - `memory/<skill_id>`
  - `memory/employees/<employee_bucket>/skills/<skill_id>`
- Include read-only legacy import candidates for workspace group-run memory:
  - `<work_dir>/openclaw/<employee_id>/memory/<skill_id>`
- Include IM role memory as a source candidate:
  - `<legacy_bucket>/roles/<role_id>/MEMORY.md`

## Non-Goals

- Do not move or rewrite existing memory files.
- Do not inject profile instructions, project memory, or IM role memory into desktop prompt.
- Do not change `MemoryTool` write target.
- Do not create profile home directories automatically.
- Do not delete legacy `employee_id + skill_id` buckets.
- Do not preserve OpenClaw-shaped directories as a compatibility target for new writes. They are migration inputs only.

## Implementation Checklist

- `[x]` Add `ProfileMemoryLocator` data shape.
- `[x]` Add `build_profile_memory_locator`.
- `[x]` Keep `build_memory_dir_for_session` output unchanged.
- `[x]` Add compile-level tests for profile home target and legacy read-candidate order.
- `[x]` Thread `profile_id` through session execution context, turn execution context, and runtime tool setup.
- `[x]` Wire locator into runtime tool setup while registering `MemoryTool` against the legacy bucket.
- `[x]` Add `ProfileMemoryBundle` reader: prefer profile `MEMORY.md`, then fall back through legacy candidates.
- `[x]` Wire profile memory bundle content into prompt assembly while preserving legacy fallback when profile memory is absent.
- `[x]` Add read-only `ProfileMemoryStatus` helper for Employee Growth Workbench/Profile Home UI: reports profile file existence, active source, active source path, and legacy candidate existence without creating or rewriting files.
- `[x]` Expose read-only `get_employee_profile_memory_status` Tauri command for the future employee detail/Profile Home status bar. The command resolves profile aliases when possible and returns normalized string paths for UI display.
- `[x]` Add read-only Employee Hub Profile Home status bar that calls `get_employee_profile_memory_status` and displays active source, inspected skill, profile file existence, and legacy candidate count.
- `[r]` Read-only Employee Hub Growth Review inbox shell was removed on 2026-05-07. It created the wrong default product direction for Hermes parity; the Profile Home status bar remains.
- `[ ]` Later slice: add provenance-aware copy/import from legacy buckets into profile home.

## Verification Evidence

- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib profile_memory_locator --no-run`: PASS; tests compile into the Tauri libtest binary. Direct execution still depends on the known Windows `runtime_lib` libtest caveat.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib profile_memory_bundle --no-run`: PASS; tests compile into the Tauri libtest binary.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib profile_memory_status --no-run`: PASS; tests compile into the Tauri libtest binary. Direct execution still exits with the known Windows `STATUS_ENTRYPOINT_NOT_FOUND` libtest caveat.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib collect_employee_profile_memory_status_prefers_profile_home --no-run`: PASS; command response helper tests compile into the Tauri libtest binary.
- `pnpm --dir apps/runtime test -- EmployeeHubView.memory-governance.test.tsx`: PASS; covers Profile Home status bar command call and rendering.
- `pnpm --dir apps/runtime test -- EmployeeHubView`: PASS; 9 files / 27 tests.
- `pnpm --dir apps/runtime exec tsc --noEmit`: PASS.
- `cargo check --manifest-path apps/runtime/src-tauri/Cargo.toml --lib`: PASS.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_chat_commands -- --nocapture`: PASS; confirms the Windows-friendly profile runtime schema, session alias persistence, and session execution context `profile_id` loading entrypoint remains executable.
- `cargo test --manifest-path packages/runtime-chat-app/Cargo.toml --test execution_context`: PASS.
- `cargo test --manifest-path packages/runtime-chat-app/Cargo.toml --test execution_assembly`: PASS.
- `cargo test --manifest-path packages/runtime-chat-app/Cargo.toml --test preparation`: PASS.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib --no-run`: PASS.
- `pnpm test:rust-fast`: PASS.
