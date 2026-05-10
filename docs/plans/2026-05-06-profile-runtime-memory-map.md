# Profile Runtime Memory Map

Phase: Self-Improving Profile Runtime / Phase 0 baseline research.

This is a read-only dependency map for the current memory path, chat runtime, prompt assembly,
compaction, session transcript, and IM memory surfaces before the Memory OS refactor.

## Scope

Scanned code and docs:

- `apps/runtime/src-tauri/src/commands/chat.rs`
- `apps/runtime/src-tauri/src/commands/chat_session_io/*`
- `apps/runtime/src-tauri/src/agent/compactor.rs`
- `apps/runtime/src-tauri/src/agent/context.rs`
- `apps/runtime/src-tauri/src/agent/runtime/*`
- `apps/runtime/src-tauri/src/im/*`
- `apps/runtime/src-tauri/src/commands/employee_agents/*`
- `packages/runtime-chat-app/*`
- `docs/architecture/*memory*`, `docs/plans/*memory*` filename surfaces; no dedicated `*memory*`
  architecture or plan files were found beyond memory mentions in broader docs.
- Related docs: `docs/architecture/employee-identity-model.md`,
  `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`,
  `docs/plans/2026-04-06-desktop-local-chat-runtime-kernel-design.md`,
  `docs/plans/2026-04-08-workclaw-session-spine-design.md`.

Out of scope for this research batch:

- Runtime code changes.
- Memory directory migration.
- Prompt behavior changes.
- New memory tools.
- Roadmap status edits.

## Current Memory Flow

Current root paths are centralized in `RuntimePaths::new`:

- `runtime root/memory`: long-term memory root.
- `runtime root/transcripts`: compaction JSONL transcripts.
- `runtime root/sessions/<session_id>`: session journal events/state/transcript markdown.

Desktop chat memory path is decided by:

- `runtime_chat_app::preparation::resolve_memory_bucket_employee_id`: returns
  `ChatExecutionContext.employee_id`.
- `PoolChatSettingsRepository::load_session_execution_context`: loads `sessions.employee_id`
  and `sessions.work_dir`.
- `prepare_local_chat_turn` in `agent/runtime/kernel/turn_preparation.rs`: resolves the memory
  bucket employee id and stores it in `ExecutionContext.memory_bucket_employee_id`.
- `prepare_routed_prompt` in `agent/runtime/kernel/routed_prompt.rs`: reuses
  `ExecutionContext.memory_bucket_employee_id` for prompt-routed skills.
- `setup_runtime_tool_registry` in `agent/runtime/kernel/tool_registry_setup.rs`: builds a
  `ProfileMemoryLocator`, registers ordinary `MemoryTool` operations against
  `profiles/<profile_id>/memories` when a profile exists, and falls back to the legacy bucket only
  when no profile is available.
- `build_memory_dir_for_session` in `agent/runtime/runtime_io/runtime_support.rs`:
  - no employee id: `<memory_root>/<skill_id>`
  - employee id present: `<memory_root>/employees/<employee_bucket>/skills/<skill_id>`
  - `employee_bucket` is lowercase and non-alphanumeric characters collapse to `_`.

Important mismatch: `setup_runtime_tool_registry` currently uses `app.path().app_data_dir()` as the
memory root, while employee memory management commands use `runtime_paths.memory_dir`. Existing tests
and docs describe `memory/<skill_id>` and `memory/employees/<employee_bucket>/skills/<skill_id>`.
This should be verified before migration because the registry path may rely on legacy app data root
behavior or may be an accidental root-level difference.

Memory creation, read, write, and delete surfaces:

- `MemoryTool::view/add/replace/remove/history` operate on the configured profile directory's
  `MEMORY.md` and `history.jsonl`; `scope=project` targets
  `PROJECTS/<workspace_hash>.md` and `project-history.jsonl`; `remove` requires `confirm=true`.
- `MemoryTool::write` creates its configured directory and writes `<key>.md`.
- `MemoryTool::read/list/delete` read, enumerate, or delete `<key>.md` under the configured bucket.
- Prompt injection reads only `MEMORY.md` through `load_memory_content(memory_dir)`.
- `MemoryTool::capture_im` delegates to `im::memory::capture_entry`.
- `im::memory::capture_entry` creates/appends:
  - `daily/<yyyy-mm-dd>.md`
  - `sessions/<thread_id>.md`
  - `roles/<role_id>/MEMORY.md`, only when `confirmed && confidence >= 0.7`
  - `org/CASEBOOK.md`, only for confirmed fact/decision/rule categories.
- `im::memory::recall_context` reads `roles/<role_id>/MEMORY.md`,
  `sessions/<thread_id>.md`, and `org/CASEBOOK.md`.
- `employee_agents::memory_commands` stats/export/clear operate under
  `runtime_paths.memory_dir/employees/<employee_bucket>/skills`.
- `group_run_execution_service::execute_group_step_in_employee_context_with_pool` registers a
  separate `MemoryTool` at either temp `workclaw-group-run-memory/<skill_id>` or
  `<session_work_dir>/openclaw/<employee_id>/memory/<skill_id>`. After the 2026-05-07 Hermes pivot,
  this path is a legacy import source only, not a target for new writes or mirror compatibility.

Session identity persistence surfaces:

- `chat_session_io::create_session_with_pool` stores `sessions.skill_id`,
  `sessions.employee_id`, `sessions.work_dir`, `sessions.session_mode`, and `sessions.team_id`.
- IM session creation in `im::agent_session_runtime::create_agent_route_session` stores
  `employee.primary_skill_id` fallbacking to `builtin-general`, employee default work dir, and
  `employee.employee_id`.
- IM binding then calls `update_session_employee_id` again to repair/reaffirm `sessions.employee_id`.
- Group run records store assignee/dispatch employee ids and step session ids, but group step memory
  path is currently workspace-local rather than the desktop runtime memory root.

## Runtime Context Injection Points

The main prompt assembly chain is:

1. `chat.rs::send_message` persists the user message into `messages` with `content_json` for parts.
2. `SessionRuntime::run_send_message` delegates to `task_entry::run_and_finalize_primary_local_chat_task`.
3. Turn preparation reconstructs history via `RuntimeTranscript::reconstruct_history_messages_with_runtime_paths`
   and appends the current turn message.
4. `prepare_runtime_tools`:
   - registers runtime tools, including `memory`, `skill`, `compact`, browser, search, etc.
   - builds workspace skill context through `build_workspace_skill_context`.
   - loads memory with `chat_io::load_memory_content(&registry_setup.memory_dir)`.
   - builds `ContextBundle`.
5. `ContextBundle::build` calls `runtime_chat_app::build_system_prompt_sections` and
   `compose_system_prompt_from_sections`.

Current system prompt section order is:

1. Base skill prompt.
2. Runtime capability snapshot: work dir, tool names, model name, max iterations.
3. Workspace skills prompt, wrapped in the mandatory skill-loading instruction.
4. Employee collaboration guidance, when `employee_id` maps to a team/candidate list.
5. Persistent memory: `---\n持久内存:\n<content>`.
6. Temporal execution guidance.
7. Tool runtime notes.
8. Runtime notes, currently labelled as search status even though continuation notes can also be
   merged into this vector.

Current memory injection is therefore one-file and bucket-local:

- Only `MEMORY.md` at the selected memory bucket root is injected automatically.
- Profile instructions are not loaded from the new Profile Home yet.
- `PROJECT_MEMORY.md` is not loaded.
- IM `roles/<role_id>/MEMORY.md` is not automatically injected into desktop chat unless a caller
  explicitly uses `memory.recall_im` or separately writes bucket-root `MEMORY.md`.

Best future injection boundary:

- Keep the final injection at `ContextBundle`/`build_system_prompt_sections`, because this is where
  memory, skills, temporal context, collaboration guidance, and runtime notes are already composed.
- Move loading and budgeting upstream into a profile memory service called from `prepare_runtime_tools`,
  replacing `load_memory_content(memory_dir)` with a structured memory bundle.
- Suggested section mapping:
  - `MEMORY.md`: profile-level operating memory, injected where current `持久内存` lives.
  - `instructions/USER_CONTEXT.md`: user/team context instructions, separate from Memory OS.
  - `PROJECT_MEMORY.md`: workspace-scoped memory, injected after capability/workdir context and
    before task execution guidance; selection should be keyed by normalized workspace hash.

## Compaction And Transcript Flow

There are two compaction paths.

Manual compaction:

- `chat.rs::compact_context` calls `chat_compaction::compact_context_with_pool`.
- `chat_session_io::load_compaction_inputs_with_pool` loads DB `messages`, normalizing assistant
  structured content and rendering user attachment parts.
- `run_compaction` saves the original messages to `runtime_paths.transcripts_dir`.
- `auto_compact` asks the configured model for a summary.
- `chat_session_io::replace_messages_with_compacted_with_pool` deletes the original DB messages and
  inserts two compacted messages:
  - a user message containing `[对话已压缩。完整记录: <path>]` plus summary/context.
  - an assistant acknowledgement.

Auto compaction during a live run:

- `AgentExecutor::execute_turn_impl` estimates token count and calls
  `compaction_pipeline::maybe_auto_compact`.
- `compaction_pipeline::run_compaction` calls `compactor::save_transcript`, then `auto_compact`.
- `compactor::save_transcript` writes JSONL lines into `runtime_paths.transcripts_dir` using
  `<session_id>_<timestamp>.jsonl`.
- `auto_compact` adds limited rehydration hints from recent file-tool entries, then replaces the
  in-memory message list for that model turn.
- `attempt_runner` converts the compaction outcome into a `TurnCompactionBoundary`.
- `TurnStateSnapshot` stores transcript path, original tokens, compacted tokens, summary, and
  reconstructed history length.
- `OutcomeCommitter` persists only the assistant terminal result to `messages`; it does not replace
  DB history with compacted messages during auto compaction.
- `SessionJournalStore::append_event` writes `events.jsonl`, projected `state.json`, and
  `transcript.md` under `runtime_paths.sessions_dir/<session_id>`.

Restore/display surfaces:

- Normal chat UI messages come from `chat_session_io::get_messages_with_pool`, which reads DB
  `messages` and normalizes assistant structured content.
- Runtime history for the next turn is reconstructed from DB `messages` by `RuntimeTranscript`.
- Recent compaction continuity is recovered from session journal state by
  `turn_preparation::load_recent_continuation_context`,
  `resolve_recent_compaction_runtime_notes`, and
  `resolve_compaction_continuation_preference`.
- Export uses `chat_session_io::export_session_markdown_with_pool`, combining DB messages,
  `session_run_events`, and session journal state; it renders compaction boundary path/summary under
  recovered run sections.

Implication for profile sessions:

- Existing transcript artifacts are split across DB `messages`, SQLite `session_run_events`,
  journal files under `sessions/<session_id>`, and compaction JSONL files under `transcripts`.
- Phase 2 `profiles/<profile_id>/sessions` should not replace all of these at once. The first
  profile-facing slice now writes `profiles/<profile_id>/sessions/<session_id>/manifest.json` with
  session id, skill id, workdir, source, timestamp, journal path, state path, transcript path, and
  the latest run summary parsed from `state.json` when available. It also reads `events.jsonl`
  `tool_completed` records into bounded `tool_summaries` and `state.json` turn-state compaction
  metadata into `compaction_boundaries`.
- The 2026-05-07 follow-up slice now writes these bounded manifest fields into
  `profile_session_index` and `profile_session_fts`; `search_profile_sessions` can search by
  `profile_id` and query. FTS5 remains the primary index, with a profile-scoped `LIKE` fallback for
  Chinese phrases that `unicode61` does not tokenize into searchable terms.
- The 2026-05-08 follow-up slice adds run-level FTS documents with `document_kind=run` and
  `run_id`, sourced from `session_runs` plus linked DB `messages`. Search results now include
  `document_kind` and `matched_run_id`, so profile recall can point to the specific turn that
  matched instead of only the aggregate session.
- The 2026-05-08 filter slice adds shared search filters for `work_dir`/`workspace`,
  `updated_after`, `updated_before`, `skill_id`, and manifest `source`. These filters apply to
  empty-query listing, FTS search, and Chinese `LIKE` fallback.

## IM Memory Boundary

IM and desktop chat currently share some infrastructure but use two logical memory models.

Shared parts:

- IM-created sessions are normal `sessions` rows with `skill_id`, `employee_id`, `work_dir`,
  `session_mode`, and model id.
- IM reply execution ultimately uses the same chat/runtime execution stack when bridged into a
  session.
- If the session has an employee id, the runtime `memory` tool is registered under the same
  employee/skill bucket shape used by desktop chat.

Separate parts:

- Desktop prompt injection reads the active profile `MEMORY.md` and current workspace
  `PROJECTS/<workspace_hash>.md` when present, applies a default character budget, and falls back to
  legacy bucket memory when profile memory is absent.
- Generic desktop memory tool operations now prefer profile `view/add/replace/remove/history` on
  `MEMORY.md`, or on `PROJECTS/<workspace_hash>.md` when `scope=project`; legacy
  read/write/list/delete flat `<key>.md` files remain available.
- IM capture/recall uses `daily`, `sessions`, `roles`, and `org` subdirectories inside that bucket.
- `recall_im` returns role/session/org content only when the model calls the tool.
- Confirmed IM long-term memory lands in `roles/<role_id>/MEMORY.md` and `org/CASEBOOK.md`, not in
  bucket-root `MEMORY.md`.

IM routing identity surfaces:

- `im::resolve_agent_id` prefers explicit `openclaw_agent_id`, then `employee_id`, then `role_id`.
- `agent_conversation_bindings` uses `agent_id`.
- Legacy `im_thread_sessions` uses `employee_id` in its primary key with `thread_id`.
- `employee_agents::session_service` converts between `AgentSession` and employee-facing structs,
  keeping `employee_id` as the public alias.

Conclusion: IM memory and desktop chat memory are not fully separate storage roots, but they are two
different models inside or near the same bucket. A profile migration must make the IM thread/role/org
model first-class instead of assuming `MEMORY.md` covers it.

## Profile Memory Target Shape

Roadmap target:

```text
profiles/<profile_id>/
  config.json
  instructions/
    RULES.md
    PERSONA.md
    USER_CONTEXT.md
  memories/
    MEMORY.md
    PROJECTS/<workspace_hash>.md
  skills/
  sessions/
  curator/
  growth/
```

Recommended migration-aware shape:

- `profile_id` should become the runtime identity for new memory, session index, growth, curator,
  and skill lifecycle behavior.
- `employee_id` remains a UI/API/routing alias during migration.
- `skill_id` remains execution metadata and skill-source metadata, but should not be the memory root.
- Legacy memory buckets remain readable through a fallback:
  - general legacy: `memory/<skill_id>`
  - employee legacy: `memory/employees/<employee_bucket>/skills/<skill_id>`
  - IM legacy submodel: `daily`, `sessions`, `roles`, `org` under the legacy bucket.

Calling chains affected by `profiles/<profile_id>/memories`:

- Session creation and loading:
  - `chat.rs::create_session`
  - `chat_session_io::create_session_with_pool`
  - `PoolChatSettingsRepository::load_session_execution_context`
  - IM session creation in `im::agent_session_runtime`
  - employee group step session creation and execution.
- Prompt preparation:
  - `runtime_chat_app::prepare_execution_context`
  - `resolve_memory_bucket_employee_id`
  - `prepare_local_chat_turn`
  - `prepare_routed_prompt`
  - `prepare_runtime_tools`
  - `setup_runtime_tool_registry`
  - `ContextBundle::build`
  - `compose_system_prompt_from_sections`.
- Memory tool and commands:
  - `MemoryTool::new/execute`
  - `im::memory::{memory_paths,capture_entry,recall_context}`
  - `employee_agents::{get/export/clear}_employee_memory`
  - `group_run_execution_service` memory registration.
- Session and transcript:
  - `RuntimeTranscript`
  - `compaction_pipeline`
  - `compactor::save_transcript`
  - `SessionJournalStore`
  - `chat_session_io::export_session_markdown_with_pool`.

## Session Search Index Candidates

Current legacy search is `search_sessions_global_with_pool`, a SQL `LIKE` over `sessions.title` and
`messages.content`. It has no profile filter, workspace filter, skill-source filter, or tool-summary
recall.

The new profile search entrypoint is `search_profile_sessions`, backed by
`profile_session_index`/`profile_session_fts`. Current coverage is manifest-backed: session id,
profile id, skill id, workdir, latest run summary, bounded tool summaries, and compaction boundary
summaries. It now also indexes DB `messages` user text and assistant final text, including common
structured assistant JSON `text` fields. It also writes run-level FTS documents from `session_runs`
and linked messages, returning `document_kind` and `matched_run_id` for precise turn recall. It does
not yet index IM thread payloads or route/debug metadata.

The runtime `memory` tool now exposes `action=search`; when a profile id is available, tool setup
injects the SQLite pool and current profile id so the agent can recall these profile-scoped session
summaries directly during a task. Sessions without a profile keep the old memory behavior and report
that Profile Session Search is not configured if `search` is requested.
`memory.search` can pass the same filters as the Tauri command: `work_dir` or `workspace`,
`updated_after`, `updated_before`, `skill_id`, and manifest `source`.
Profile and project memory mutations now also create version snapshots under `versions/profile/`
or `versions/projects/<workspace_key>/`. The `memory` tool supports `versions`, `view_version`, and
confirmed `rollback`, so ordinary self-improving memory writes are auditable and reversible without
introducing a default approval queue.
Profile session indexing also writes a profile-local transcript mirror at
`profiles/<profile_id>/sessions/<session_id>/transcript.md`, combining DB messages, matched run ids,
tool summaries, and compaction boundaries into a readable evidence artifact.

After a primary task terminal outcome is committed, WorkClaw refreshes the current session's profile
index from the `sessions` row and DB `messages`, so the assistant's final answer is searchable by
subsequent turns without waiting for a later tool setup pass.

Phase 2 `session_search` should index:

- `sessions`:
  - `id`, `title`, `created_at`, `skill_id`, `employee_id`, future `profile_id`, `work_dir`,
    `session_mode`, `team_id`, model id.
- `messages`:
  - user text content.
  - assistant final text.
  - rendered `content_json` for user parts/attachments where safe.
  - assistant structured `items` text/tool summaries.
- `session_runs`:
  - run id, status, user/assistant message ids, buffered text, error kind/message, timestamps.
- `session_run_events`:
  - `tool_started`: tool name and normalized input summary.
  - `tool_completed`: tool name, output summary, error flag.
  - `run_stopped`/`run_failed`: stop reason and error summary.
  - `skill_route_recorded`: selected runner, selected skill, fallback reason, tool plan summary.
- Session journal files:
  - `state.json` for turn state, invoked skills, compaction boundary, reconstructed history length.
  - `transcript.md` for human-readable run transcript.
- Compaction transcripts:
  - JSONL path and summary from `TurnCompactionBoundary`.
  - Do not blindly index full transcript bodies by default; they can be large and may duplicate
    message/tool content. Store path plus compact summary first.
- IM bindings:
  - `agent_conversation_bindings` and `im_thread_sessions` fields for channel/account/thread/topic,
    `agent_id`/`employee_id`, session key, and conversation id.
- Group run data:
  - `group_runs`, `group_run_steps`, `group_run_events` for multi-employee task identity,
    assignee, dispatch source, outputs, review/reassignment events.
- Route diagnostics:
  - `route_attempt_logs` as optional ranking/debug metadata, not primary searchable content.

The initial FTS row should probably represent a session-turn or run-level document, not only a whole
session, so tool calls and final results can be searched without overloading one large session blob.

## Migration Risks

- Root mismatch risk: runtime tool registration uses `app.path().app_data_dir()` as memory root,
  while runtime path governance and employee memory commands use `runtime_paths.memory_dir`.
- Prompt regression risk: changing the memory loader can reorder prompt sections or inject more
  content than current models expect.
- IM memory invisibility: confirmed IM memory in `roles/<role_id>/MEMORY.md` is not automatically
  injected today. Treating bucket-root `MEMORY.md` as the only legacy source would lose IM learning.
- Two memory write models: flat key files and IM daily/session/role/org files have different
  semantics and risk assumptions.
- Destructive cleanup risk: `clear_employee_memory_from_root` deletes skill buckets. Profile
  migration must not redirect this to profile home without explicit UI/risk-confirmation semantics.
- Group-run divergence: group step execution currently uses workspace-local `openclaw/<employee>/memory`,
  not the desktop runtime memory root. Treat it as legacy import state only.
- Legacy schema risk: startup-critical queries assume `sessions.employee_id`, `session_mode`,
  `team_id`, and IM binding columns with fallback helpers in some places. Profile columns need
  legacy fallbacks and regression tests.
- Compaction artifact risk: auto compaction records only turn-state boundary metadata in the journal;
  manual compaction mutates DB messages. Migration must preserve both meanings.
- `.skillpack` boundary risk: profile skills and memory growth must not mutate encrypted skillpack
  contents or treat them as resettable preset skills.

## Open Questions

- Should `profile_id` reuse `employee_id`, or should it be a stable UUID with `employee_id` as an
  alias? The roadmap leaves this unresolved.
- Is the `app.path().app_data_dir()` memory root in `setup_runtime_tool_registry` intentional, or
  should it have been `runtime_paths.memory_dir`?
- Should legacy IM `roles/<role_id>/MEMORY.md` be mapped into profile `MEMORY.md`, into a separate
  IM memory namespace, or into `PROJECTS/<workspace_hash>.md` when a thread is workspace-specific?
- Should profile instructions include a global user context overlay, or should all user/team context
  live per profile?
- What is the default memory injection budget, and is it static or model-context-aware?
- Should manual compaction continue to rewrite DB `messages`, or should future compaction only
  create profile session artifacts plus index rows?
- How should group-run workspace-local memory be reconciled with profile homes when the same employee
  works across multiple work dirs?
- How much IM thread content should be indexed for `session_search` by default, especially for
  external channels with privacy expectations?

## Recommended First Implementation Slice

Start with a migration-aware profile memory locator and read path, without changing prompt output:

1. Add a `ProfileMemoryLocator`/service that accepts `{profile_id?, employee_id?, skill_id, work_dir}`
   and returns:
   - future profile memory paths,
   - legacy employee/skill bucket paths,
   - legacy general skill bucket paths,
   - IM submodel paths.
2. Add tests that prove the resolver preserves current paths for:
   - no employee id: `memory/<skill_id>`
   - employee id: `memory/employees/<employee_bucket>/skills/<skill_id>`
   - IM role memory under the same legacy bucket.
3. Wire the service into a read-only path behind existing behavior first: keep injecting the exact
   current bucket-root `MEMORY.md`, but make the path decision observable and testable.
4. Only after this migration layer is stable, add profile `MEMORY.md` with legacy fallback and an
   explicit injection budget.
5. Defer writes, `memory` tool replacement, and directory migration until after the read path has
   legacy-schema and legacy-directory regression coverage. Do not add a default approval queue as
   the first write path; the eventual write path should be Hermes-aligned direct memory operations
   with provenance, versioning, rollback, and high-risk confirmations only where needed.

This first slice advances Phase 0 and prepares Phase 2 without extending the old
`employee_id + skill_id` memory bucket as the design center.
