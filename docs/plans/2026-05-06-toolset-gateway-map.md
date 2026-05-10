# Toolset Gateway Pre-Refactor Dependency Map

## Scope

This is a Phase 0 read-only dependency map for the future Phase 7 Toolset Gateway work in `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`.

It covers the current tool registry, tool metadata, approval bus, sidecar tools, MCP bridge, browser automation, exec/shell permissions, skill-level tool filters, profile/session/runtime policy inputs, and relevant settings UI surfaces.

Primary files inspected:

- `apps/runtime/src-tauri/src/agent/registry.rs`
- `apps/runtime/src-tauri/src/agent/tool_manifest.rs`
- `apps/runtime/src-tauri/src/agent/tools/*`
- `apps/runtime/src-tauri/src/agent/permissions.rs`
- `apps/runtime/src-tauri/src/agent/approval_flow.rs`
- `apps/runtime/src-tauri/src/approval_bus.rs`
- `apps/runtime/src-tauri/src/approval_rules.rs`
- `apps/runtime/src-tauri/src/commands/mcp.rs`
- `apps/runtime/sidecar/src/*`
- `packages/runtime-policy/*`
- `docs/browser-automation-integration.md`
- `apps/runtime/src/components/settings/*`

No runtime code or permission policy was changed.

## Current Tool Registry And Metadata

`ToolRegistry` in `apps/runtime/src-tauri/src/agent/registry.rs` is the central in-memory registry. It stores `Arc<dyn Tool>` by `Tool::name()`, supports register/unregister/get, returns raw tool definitions for model providers, and projects `ToolManifestEntry` by combining `Tool::description()` with `Tool::metadata()`.

Static standard tools are registered by `ToolRegistry::with_standard_tools()`:

- `read_file`, `write_file`, `glob`, `grep`, `edit`
- `list_dir`, `file_stat`, `file_delete`, `file_move`, `file_copy`
- `todo_write`, `web_fetch`, `exec`, `bash`
- `screenshot`, `open_in_folder`

Runtime setup expands this surface in `apps/runtime/src-tauri/src/agent/runtime/kernel/tool_registry_setup.rs` and the older helper in `apps/runtime/src-tauri/src/agent/runtime/tool_registry_builder.rs`:

- Replaces/augments shell tools with process-manager backed `bash`, `exec`, `bash_output`, `bash_kill`, `exec_output`, `exec_kill`.
- Registers browser sidecar tools through `register_browser_tools(...)` and OpenClaw-compatible `browser`.
- Registers aliases `read -> read_file`, `find -> glob`, `ls -> list_dir`.
- Registers runtime support tools: `task`, `clawhub_search`, `clawhub_recommend`, `github_repo_download`, `employee_manage`, `vision_analyze`, `document_analyze`, `memory`, `skill`, `compact`, `ask_user`.
- Registers `web_search` from a configured search provider, or a synthetic MCP-backed `web_search` fallback if a search-like MCP tool exists.

`ToolMetadata` in `apps/runtime/src-tauri/src/agent/tool_manifest.rs` has:

- `display_name`
- `category`: `file`, `shell`, `web`, `browser`, `system`, `planning`, `agent`, `memory`, `search`, `integration`, `other`
- `read_only`
- `destructive`
- `concurrency_safe`
- `open_world`
- `requires_approval`
- `source`: `native`, `runtime`, `sidecar`, `mcp`, `plugin`, `alias`

Current metadata coverage is partial:

| Tool or family | Current metadata facts |
| --- | --- |
| `read_file` | `category=file`, `read_only=true`, default `source=native` |
| `write_file` | `category=file`, `destructive=true`, `requires_approval=true`, default `source=native` |
| `edit` | `category=file`, `destructive=true`, `requires_approval=true`, default `source=native` |
| `bash` | `category=shell`, `destructive=true`, `requires_approval=true`, `source=runtime` |
| `exec` | `category=shell`, `destructive=true`, `requires_approval=true`, `source=runtime` |
| `web_fetch` | `category=web`, `read_only=true`, `open_world=true`, default `source=native` |
| `web_search` | `category=search`, `read_only=true`, `open_world=true`, default `source=native` in the tool implementation |
| `document_analyze` | `category=file`, `read_only=true`, `source=runtime` |
| `vision_analyze` | `category=other`, `read_only=true`, `source=runtime` |
| `SidecarBridgeTool` browser/MCP tools | no metadata override, so currently default to `category=other`, `source=native`, not `browser/sidecar` or `mcp` |
| `file_delete`, `file_move`, `file_copy`, `screenshot`, `open_in_folder`, `task`, `skill`, `memory`, `ask_user`, `employee_manage`, aliases | no explicit metadata override observed, so they inherit default `other/native` unless wrapped by tests/fakes |

Important: `requires_approval` metadata is not the approval source of truth today. Actual approval decisions come from `packages/runtime-policy/src/permissions.rs` via `PermissionMode::decision(...)` and `classify_action_risk(...)`.

Tool execution path:

1. Runtime builds or updates the registry.
2. `prepare_runtime_tools(...)` resolves the effective tool plan.
3. `CapabilitySnapshot` exposes active tool definitions to the model.
4. `dispatch_tool_call(...)` emits `ToolStarted`, checks allowlist/effective plan, asks approval if policy says `Ask`, then calls `Tool::execute(...)` in `run_tool(...)`.
5. Tool result is emitted as `tool-call-event` and appended as `SessionRunEvent::ToolCompleted`.

## Approval Flow Map

Current approval is split into permission classification, approval request creation, resolution, reusable rules, and restart recovery.

Decision entrypoint:

- `dispatch_tool_call(...)` in `apps/runtime/src-tauri/src/agent/runtime/tool_dispatch.rs`
- It calls `resolve_dispatch_permission_decision(...)`.
- The decision first checks `allowed_tools` or `effective_tool_plan.allowed_tools`; disallowed tools return `Deny` before execution.
- If allowed by scope, it calls `PermissionMode::decision(...)`.

Permission classification:

- `packages/runtime-policy/src/permissions.rs`
- `PermissionMode::Unrestricted` allows all.
- `PermissionMode::Default` and `AcceptEdits` ask only when `classify_action_risk(...) == Critical`.
- Critical today includes `file_delete`, `exec`, risky `write_file`/`edit`, risky `bash`, `browser_evaluate`, and some browser click/type/press/act patterns.
- Normal includes most read tools, `web_search`, `web_fetch`, and many browser navigation/snapshot/state actions.

Approval request:

- `gate_tool_approval(...)` in `apps/runtime/src-tauri/src/agent/runtime/approval_gate.rs`
- If app/session state is available and `approval_bus_v1` is enabled, it first checks `approval_rules`.
- If no reusable rule matches, it calls `request_tool_approval_and_wait(...)`.
- If approval bus is disabled or app/session state is missing, it falls back to manual in-memory confirmation with a 15 second timeout.

Approval persistence and notifications:

- `request_tool_approval_and_wait(...)` creates a pending row through `ApprovalManager::create_pending_with_pool(...)`.
- The row stores `session_id`, `run_id`, `call_id`, `tool_name`, `input_json`, `summary`, `impact`, `irreversible`, and `resume_payload_json`.
- It emits `approval-created` and `tool-confirm-event`.
- It notifies registered IM hosts through `maybe_notify_registered_approval_requested_with_pool(...)`.

Resolution:

- Desktop command: `resolve_approval(...)` in `apps/runtime/src-tauri/src/commands/approvals.rs`.
- IM commands are parsed through Feishu-compatible approval command handling and unified IM host inbound handling.
- `ApprovalManager::resolve_with_pool(...)` updates only `status='pending'`, so first resolver wins.
- `allow_always` persists an `approval_rules` record through `persist_allow_always_rule_with_tx(...)`.

Reusable approval rules:

- `apps/runtime/src-tauri/src/approval_rules.rs`
- Rule matching is based on `runtime_policy::approval_rule_fingerprint(...)`.
- Fingerprints currently exist for:
  - `file_delete`: path + recursive flag
  - critical `bash`: normalized command
  - `exec`: normalized command
- Browser approvals and write/edit approvals currently do not get reusable fingerprints.

Restart recovery:

- `spawn_approval_recovery_bootstrap(...)` starts recovery from `lib.rs`.
- `recover_approved_pending_work_with_pool(...)` finds `status='approved' AND resumed_at IS NULL`.
- It skips replay when the matching `ToolCompleted` event already exists.
- Otherwise it reconstructs `ApprovalResumePayload`, runs the tool from the registry with a minimal `ToolContext`, appends `ToolCompleted`, then appends `RunFailed` with `error_kind='approval_recovery'`.
- It marks `resumed_at` afterward.

## Sidecar And MCP Tool Boundary

Sidecar HTTP server:

- `apps/runtime/sidecar/src/index.ts`
- Runs on `http://localhost:8765` by default.
- Exposes browser, MCP, web search, channel adapter, OpenClaw route, and Feishu compatibility endpoints.

Browser:

- `apps/runtime/sidecar/src/browser.ts` owns Playwright control.
- `apps/runtime/src-tauri/src/agent/tools/browser_tools.rs` registers 17 `browser_*` tools as `SidecarBridgeTool`.
- `apps/runtime/src-tauri/src/agent/tools/browser_compat.rs` registers the OpenClaw-style `browser` tool.
- `docs/browser-automation-integration.md` documents P0 support for `browser_launch`, `browser_snapshot`, `browser_act`, legacy `browser_*`, and the unified `browser(action=...)` compat tool.

MCP:

- `apps/runtime/src-tauri/src/commands/mcp.rs` persists configured MCP servers in SQLite and asks sidecar to connect.
- Sidecar `MCPManager` in `apps/runtime/sidecar/src/mcp.ts` starts MCP servers over stdio and calls/list tools via the MCP SDK.
- Runtime registers MCP tools as `mcp_<server>_<tool>` using `SidecarBridgeTool::new_mcp(...)`.
- Runtime can also select a search-like MCP tool as a fallback `web_search`.

Boundary risks observed:

- `SidecarBridgeTool::new_mcp(...)` posts camelCase keys `serverName` and `toolName`, while sidecar `/api/mcp/call-tool` reads snake_case `server_name` and `tool_name`.
- `add_mcp_server_with_registry(...)` and `restore_saved_mcp_servers(...)` post `{ name, config: { command, args, env } }`, while sidecar `/api/mcp/add-server` reads top-level `command`, `args`, and `env`.
- Runtime `/api/mcp/list-tools` calls use `serverName`, while sidecar reads `server_name`.
- Because `SidecarBridgeTool` does not override metadata, browser and MCP tools are not accurately visible to the manifest as `source=sidecar/mcp` or `category=browser/integration/search`.

IM and channels:

- Sidecar channel adapters live under `apps/runtime/sidecar/src/adapters/*`.
- Runtime IM host logic lives under `apps/runtime/src-tauri/src/commands/im_host/*`, `feishu_gateway/*`, and `wecom_gateway/*`.
- Approval notifications and resolution notices are routed to registered IM hosts, but IM capabilities are not represented as agent tools in the same way browser/MCP are.

Settings UI:

- `apps/runtime/src/components/settings/mcp/*` invokes `list_mcp_servers`, `add_mcp_server`, and `remove_mcp_server`.
- `apps/runtime/src/components/settings/desktop/*` controls `runtime_operation_permission_mode` as `standard` or `full_access`.
- `apps/runtime/src/components/settings/channels/*` displays channel lifecycle phases such as `awaiting_approval` and `approval_resolved`.

## Proposed Toolset Mapping

This is a mapping proposal only; no gateway implementation was made.

| Toolset | Existing tools/surfaces to map |
| --- | --- |
| `core` | `read_file`, `glob`, `grep`, `list_dir`, `file_stat`, `todo_write`, `compact`, `ask_user`, aliases `read/find/ls` |
| `memory` | Current `memory` tool and legacy `build_memory_dir_for_session(...)` bucket; future Profile Runtime memory tools should move here |
| `skills` | `skill`, skill command dispatch, `clawhub_search`, `clawhub_recommend`, `github_repo_download`, skill library/install flows |
| `web` | `web_fetch`, `web_search`, search providers, MCP search fallback |
| `browser` | `browser`, all `browser_*` sidecar tools, browser compat upload/staging |
| `im` | Feishu/WeCom channel adapters, IM host commands, approval notification/resolve commands, route bridge |
| `desktop` | `bash`, `exec`, process manager tools, `screenshot`, `open_in_folder`, file mutation tools, desktop runtime lifecycle/open path operations |
| `media` | `vision_analyze`, `document_analyze`, screenshots, future attachment/media analysis tools |
| `mcp` | `mcp_<server>_<tool>`, MCP server add/remove/list commands, MCP fallback search |

Current closest equivalent to toolsets:

- Skill frontmatter: `allowed_tools`, `denied_tools`, `allowed_tool_sources`, `denied_tool_sources`, `allowed_tool_categories`, `denied_tool_categories`, `mcp-servers`.
- Runtime app settings: `runtime_tool_policy_denied_tools`, `runtime_tool_policy_denied_categories`, `runtime_tool_policy_allowed_sources`, `runtime_tool_policy_allowed_mcp_servers`.
- Named profiles: `SafeDefault`, `Coding`, `Browser`, `Employee` in `tool_profiles.rs`.
- Session permission mode: `standard`/`full_access` in settings, mapped to runtime permission modes.

Gaps before Toolset Gateway:

- No `requires_toolsets`, `optional_toolsets`, or `denied_toolsets` fields in `SkillConfig`.
- No profile-level `allowed_toolsets`.
- No approval rule fingerprint scoped by toolset.
- No manifest-level toolset projection.
- Browser/MCP metadata is too generic to support reliable source/category/toolset filtering.

## Permission And Safety Risks

1. Metadata and approval are not unified. `requires_approval=true` exists on some tools but approval decisions do not read metadata; they read hard-coded risk classification in `runtime-policy`.

2. Several mutating tools lack metadata. `file_delete`, `file_move`, `file_copy`, `employee_manage`, and `open_in_folder` default to `other/native`, so category/source filtering cannot reliably identify them.

3. `file_move` and `file_copy` are not classified as critical in `runtime-policy`. `file_move` changes filesystem state, and `file_copy` can overwrite destinations through `fs::copy`, but neither currently triggers approval by name.

4. Browser safety is text heuristic based. `browser_click`, `browser_type`, `browser_press_key`, and `browser_act` become critical only for submit-like text, Enter, `evaluate`, or commit keywords. `browser_launch` and `browser_navigate` are currently normal in code, while `docs/browser-automation-integration.md` says they trigger confirmation.

5. MCP tool risk is opaque. Dynamically registered MCP tools do not carry accurate `source=mcp`, tool category, destructive/read-only flags, or per-server risk metadata.

6. `allow_always` only fingerprints `file_delete`, critical `bash`, and `exec`. Browser, MCP, write/edit, file_move/copy, and employee-management approvals cannot currently become precise reusable rules.

7. Approval recovery replays approved tools after restart. This is covered by tests and intentional, but it should become a toolset-aware boundary because recovery uses stored approval payload and a minimal `ToolContext`, not a fresh toolset permission pass.

8. Hidden child/session tools can run with `PermissionMode::Unrestricted` in some paths. Toolset Gateway should explicitly model whether delegation inherits, narrows, or escalates profile/session toolsets.

9. Skill prompt-following invocation may still load a skill even when declared tools do not overlap the parent scope, as long as it is not command-dispatch. This preserves prompt reuse but should be visible as a non-executable or degraded toolset state when required toolsets are introduced.

## Existing Test Coverage

Permission and approval coverage:

- `packages/runtime-policy/tests/permissions.rs`
  - default mode
  - tool name normalization
  - parent/child allowed-tool narrowing
  - dangerous bash confirmation
  - browser submit heuristics
  - out-of-workspace write risk
  - `exec`, `file_delete`, and read-only decisions

- `apps/runtime/src-tauri/tests/test_approval_bus.rs`
  - pending approval persistence and waiting status projection
  - rollout flag default/disable
  - first resolver wins
  - `allow_always` rule creation and matching
  - restart recovery replay for approved pending work

- `apps/runtime/src-tauri/src/agent/runtime/approval_gate.rs` tests
  - manual confirmation true/false behavior

- `apps/runtime/src-tauri/src/agent/runtime/tool_dispatch.rs` tests
  - disallowed tools denied before execution
  - effective plan denial reason includes policy source
  - rejected approval does not execute `file_delete`
  - skill command dispatch routes only to allowed target tools

- `apps/runtime/src-tauri/tests/test_skill_permission_narrowing.rs`
  - child skill allowed tools are narrowed
  - command dispatch is denied when parent scope blocks the target
  - prompt-following skill can load with no overlapping declared tools
  - explicit `SKILL.md` path must stay within search roots

Tool registry and manifest-adjacent coverage:

- `apps/runtime/src-tauri/src/agent/registry.rs` tests standard tool names and representative metadata for `read_file`, `write_file`, and `bash`.
- `apps/runtime/src-tauri/src/agent/tool_manifest.rs` tests manifest fallback display name and metadata serialization.
- `apps/runtime/src-tauri/src/agent/runtime/effective_tool_set.rs` tests explicit allowlists, named profiles, MCP server filtering, source/category filters, denied sources, and recommended-tool loading.
- `apps/runtime/src-tauri/src/agent/runtime/tool_profiles.rs` tests `SafeDefault`, `Coding`, `Browser`, and `Employee` profile resolution.
- `apps/runtime/src-tauri/src/agent/runtime/repo.rs` tests parsing runtime tool policy defaults from app settings.
- `apps/runtime/src-tauri/tests/test_tools_complete.rs` covers standard tools, browser registration count, process tools, and filtered definitions, but its "metadata" test checks schema shape rather than `ToolMetadata` completeness.

Sidecar/browser/MCP coverage:

- `apps/runtime/sidecar/test/browser.local-automation.test.ts` covers browser local automation and upload mapping.
- `apps/runtime/src-tauri/tests/test_e2e_flow.rs` covers MCP server persistence/list/remove and skill config MCP parsing.
- E2E smoke tests mock `list_mcp_servers` for settings navigation.

Known test gap:

- No test currently proves browser/MCP tools publish accurate `ToolMetadata`.
- No test currently catches the runtime/sidecar MCP request key mismatch.
- No test covers toolset semantics because no toolset model exists yet.
- No test asserts `file_move` or `file_copy` approval behavior.

## Open Questions

- Should toolset membership be explicit per tool, derived from metadata, or both with validation?
- Should `requires_approval` metadata become authoritative, or should `runtime-policy` remain authoritative and metadata become a projection of policy?
- How should MCP tools declare read-only/destructive/open-world risk when the MCP server only provides name, description, and input schema?
- Should `browser_navigate` and `browser_launch` be approval-gated as the browser integration doc says, or remain normal as `runtime-policy` currently implements?
- Should `file_copy` and `file_move` join the high-risk approval set by default?
- How should `employee_manage` be split between read-only actions (`list_*`) and mutating actions (`create/update/apply_profile`)?
- Should `allow_always` approvals be scoped by toolset, profile, workspace, skill, and MCP server, not only tool fingerprint?
- Should approval recovery re-check current profile/toolset policy before replay, or treat prior approval as a durable capability grant?
- How should IM actions be represented: as agent-callable tools, runtime surfaces outside tool registry, or both?
- Should aliases inherit metadata from the target tool with `source=alias`, or remain independent manifest entries?

## Recommended First Implementation Slice

The smallest useful first slice is a manifest-first, behavior-preserving Toolset Gateway foundation:

1. Add a toolset projection layer that maps existing `ToolManifestEntry` plus explicit overrides into `core`, `memory`, `skills`, `web`, `browser`, `im`, `desktop`, `media`, and `mcp`.
2. Fix or wrap metadata for sidecar browser tools, MCP tools, aliases, file mutation tools, `memory`, `skill`, `task`, and `employee_manage` so source/category/destructive/read-only facts are observable.
3. Add tests that assert representative tool-to-toolset mappings and metadata completeness without changing approval behavior.
4. Add a read-only diagnostic command or trace record that exposes the current session effective toolsets, derived from the existing effective tool plan.
5. Only after the diagnostic layer is stable, wire skill/profile `requires_toolsets` and approval-by-toolset checks.

This avoids changing runtime behavior while creating the inventory and observability needed to safely gate high-risk toolsets later.
