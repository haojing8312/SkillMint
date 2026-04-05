# Real Agent Evals Harness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a local-only, manually triggered real-agent evaluation harness for WorkClaw that runs real models against real skills/open tasks, validates success/quality/speed, and keeps sensitive skill mappings and credentials out of git.

**Architecture:** Add a git-tracked `agent-evals/` contract that stores anonymized YAML scenarios keyed by `capability_id`, plus a local-only `config.local.yaml` that maps each capability to the real workspace/skill/model configuration. Implement a Rust CLI runner inside `apps/runtime/src-tauri` that reuses WorkClaw's existing `create_session_with_pool`, `SessionRuntime::run_send_message`, `list_session_runs_with_pool`, and `export_session_run_trace_with_pool` flow through a reusable non-Tauri event sink, then wrap that CLI with a root `pnpm eval:agent-real` script and write reports under `temp/agent-evals/`.

**Tech Stack:** Rust (`sqlx`, `serde_yaml`, existing Tauri runtime modules), Node.js (`scripts/*.mjs`, `node:test`), YAML scenario/config files, existing session journal + trace export pipeline.

---

### Task 1: Add the on-disk eval contract and local-only secret boundaries

**Files:**
- Create: `D:\code\WorkClaw\agent-evals\scenarios\pm_weekly_summary_xietao_2026_03_30_2026_04_04.yaml`
- Create: `D:\code\WorkClaw\agent-evals\config\config.example.yaml`
- Create: `D:\code\WorkClaw\scripts\run-agent-evals.test.mjs`
- Modify: `D:\code\WorkClaw\.gitignore`
- Modify: `D:\code\WorkClaw\package.json`

**Step 1: Write the failing contract test**

Create `D:\code\WorkClaw\scripts\run-agent-evals.test.mjs` with a first test that expects:

```js
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");

test("agent eval contract files exist and secrets stay local-only", () => {
  const scenarioPath = path.join(
    root,
    "agent-evals",
    "scenarios",
    "pm_weekly_summary_xietao_2026_03_30_2026_04_04.yaml",
  );
  const exampleConfigPath = path.join(
    root,
    "agent-evals",
    "config",
    "config.example.yaml",
  );
  const gitignore = fs.readFileSync(path.join(root, ".gitignore"), "utf8");
  const pkg = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));

  assert.equal(fs.existsSync(scenarioPath), true);
  assert.equal(fs.existsSync(exampleConfigPath), true);
  assert.match(gitignore, /agent-evals\/config\/config\.local\.yaml/i);
  assert.match(gitignore, /agent-evals\/local\//i);
  assert.match(gitignore, /temp\/agent-evals\//i);
  assert.equal(typeof pkg.scripts?.["eval:agent-real"], "string");
});
```

**Step 2: Run test to verify it fails**

Run: `node --test scripts/run-agent-evals.test.mjs`

Expected: FAIL because the scenario file, example config, ignore rules, and script entry do not exist yet.

**Step 3: Add the contract files and ignore rules**

- Add `agent-evals/scenarios/pm_weekly_summary_xietao_2026_03_30_2026_04_04.yaml` with the anonymized scenario shape:

```yaml
id: pm_weekly_summary_xietao_2026_03_30_2026_04_04
title: 项管周报汇总-谢涛-固定日期窗口
capability_id: pm_weekly_summary

kind: real-agent
mode: implicit-skill-routing
side_effect: none
enabled: true

input:
  user_text: 获取谢涛2026年3月30日到4月4日的工作日报并汇总成简报

expect:
  route:
    family: feishu-pm
    runner_not: OpenTaskRunner

  execution:
    leaf_exit_code: 0

  structured:
    equals:
      employee: 谢涛
      start_date: 2026-03-30
      end_date: 2026-04-04
      daily_count: 6
      plan_count: 6
      report_count: 5

  output:
    contains_all:
      - 金川区域排水管网改造工程（一期）
      - 土左2025老旧小区改造
    contains_any:
      - 排污通道图纸跟进
      - 飞行数据整理

thresholds:
  pass_total_ms: 150000
  warn_total_ms: 180000
  max_turn_count: 4
  max_tool_count: 6

record_metrics:
  - selected_skill
  - selected_runner
  - route_latency_ms
  - total_duration_ms
  - leaf_exec_duration_ms
  - turn_count
  - tool_count
  - fallback_reason
```

- Add `agent-evals/config/config.example.yaml` with placeholder-only local mapping:

```yaml
runtime:
  workspace_root: D:\\code\\WorkClaw
  cargo_manifest_path: apps/runtime/src-tauri/Cargo.toml

models:
  default_profile: replace_me

providers:
  replace_me:
    provider: openai
    model: gpt-5.4
    api_key_env: OPENAI_API_KEY

artifacts:
  output_dir: D:\\code\\WorkClaw\\temp\\agent-evals

capabilities:
  pm_weekly_summary:
    workspace_root: E:\\replace-me
    entry_kind: workspace_skill
    entry_name: replace-me

diagnostics:
  export_journal: true
  export_trace: true
  export_stdout_stderr: true
```

- Update `.gitignore` to ignore:

```gitignore
agent-evals/config/config.local.yaml
agent-evals/local/
temp/agent-evals/
```

- Update `package.json` to include:

```json
"eval:agent-real": "node scripts/run-agent-evals.mjs"
```

**Step 4: Run the contract test again**

Run: `node --test scripts/run-agent-evals.test.mjs`

Expected: PASS.

**Step 5: Commit**

```bash
git add .gitignore package.json agent-evals/scenarios/pm_weekly_summary_xietao_2026_03_30_2026_04_04.yaml agent-evals/config/config.example.yaml scripts/run-agent-evals.test.mjs
git commit -m "test(eval): add real-agent eval contract fixtures"
```

### Task 2: Add typed Rust models for scenarios, local config, and reports

**Files:**
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\evals\mod.rs`
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\evals\scenario.rs`
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\evals\config.rs`
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\evals\report.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\mod.rs`

**Step 1: Write the failing Rust tests**

Inside the new `scenario.rs` and `config.rs`, add tests that parse fixture YAML and validate the fields WorkClaw needs:

```rust
#[test]
fn scenario_yaml_parses_expected_thresholds() {
    let raw = include_str!("../../../../../../../agent-evals/scenarios/pm_weekly_summary_xietao_2026_03_30_2026_04_04.yaml");
    let scenario: EvalScenario = serde_yaml::from_str(raw).expect("parse scenario");
    assert_eq!(scenario.capability_id, "pm_weekly_summary");
    assert_eq!(scenario.thresholds.pass_total_ms, 150_000);
    assert_eq!(scenario.expect.structured.equals.daily_count, Some(6));
}

#[test]
fn config_yaml_requires_local_capability_mapping() {
    let raw = r#"
runtime:
  workspace_root: D:\\code\\WorkClaw
capabilities: {}
"#;
    let err = serde_yaml::from_str::<LocalEvalConfig>(raw).expect_err("config should fail");
    assert!(err.to_string().contains("capabilities"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml agent::evals -- --nocapture`

Expected: FAIL because the modules and types do not exist yet.

**Step 3: Write the minimal typed models**

Implement the new module tree with serde-backed structs:

```rust
pub mod config;
pub mod report;
pub mod scenario;

pub use config::{CapabilityMapping, LocalEvalConfig};
pub use report::{EvalReport, EvalReportStatus};
pub use scenario::{EvalScenario, EvalThresholds};
```

Key requirements:
- `scenario.rs` must deserialize the git-tracked scenario shape.
- `config.rs` must deserialize the local-only mapping file and reject missing capability mappings.
- `report.rs` must serialize a stable `pass / warn / fail` report shape for `temp/agent-evals/...`.

**Step 4: Run tests to verify parsing passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml agent::evals -- --nocapture`

Expected: PASS for the new parser tests.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/evals/mod.rs apps/runtime/src-tauri/src/agent/evals/scenario.rs apps/runtime/src-tauri/src/agent/evals/config.rs apps/runtime/src-tauri/src/agent/evals/report.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "feat(eval): add typed scenario config and report models"
```

### Task 3: Refactor session execution so the eval harness can reuse it without a Tauri window

**Files:**
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\runtime\event_sink.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\runtime\events.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\runtime\session_runtime.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\runtime\attempt_runner.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\runtime\tool_setup.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\db.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\runtime\mod.rs`

**Step 1: Write the failing runtime reuse tests**

Add tests that prove the runner can execute with a recording sink instead of a real `AppHandle`:

```rust
#[tokio::test]
async fn recording_runtime_event_sink_captures_stream_completion() {
    let sink = RecordingRuntimeEventSink::default();
    sink.emit_stream_token(StreamToken {
        session_id: "s1".to_string(),
        token: "done".to_string(),
        done: true,
        sub_agent: false,
    }).expect("emit");

    assert_eq!(sink.stream_tokens().len(), 1);
    assert_eq!(sink.stream_tokens()[0].done, true);
}
```

Also add a small test for a new database helper:

```rust
#[tokio::test]
async fn init_db_at_path_creates_sqlite_in_custom_dir() {
    let temp = tempfile::tempdir().unwrap();
    let pool = init_db_at_dir(temp.path()).await.expect("init db");
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(count > 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml recording_runtime_event_sink init_db_at_path -- --nocapture`

Expected: FAIL because the sink abstraction and custom DB helper do not exist yet.

**Step 3: Introduce the non-Tauri runtime host abstraction**

Implement:
- `event_sink.rs` with a small trait and two implementations:

```rust
pub trait RuntimeEventSink: Send + Sync {
    fn emit_stream_token(&self, token: StreamToken) -> Result<(), String>;
    fn emit_skill_route_event(&self, event: SkillRouteEvent) -> Result<(), String>;
}
```

- A `TauriRuntimeEventSink` adapter backed by `AppHandle`.
- A `RecordingRuntimeEventSink` adapter for the eval harness and unit tests.

Then refactor:
- `SessionRuntime::run_send_message`
- route execution / skill route event emission
- any `app.path().app_data_dir()` lookup in `tool_setup.rs`

so they accept an explicit runtime host/input bundle instead of assuming a live Tauri window is present.

Add `db::init_db_at_dir(root: &Path)` so the eval harness can create a real local sqlite DB in a temp run directory without `AppHandle`.

**Step 4: Run the focused Rust tests**

Run:
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml recording_runtime_event_sink -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml init_db_at_path -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml session_runtime -- --nocapture`

Expected: PASS, with existing `SessionRuntime` behavior preserved.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/runtime/event_sink.rs apps/runtime/src-tauri/src/agent/runtime/events.rs apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs apps/runtime/src-tauri/src/agent/runtime/attempt_runner.rs apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs apps/runtime/src-tauri/src/agent/runtime/mod.rs apps/runtime/src-tauri/src/db.rs
git commit -m "refactor(eval): make session runtime reusable without tauri window"
```

### Task 4: Build the Rust CLI eval runner that reuses chat sessions, journals, and traces

**Files:**
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\evals\assertions.rs`
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\agent\evals\runner.rs`
- Create: `D:\code\WorkClaw\apps\runtime\src-tauri\src\bin\agent-evals.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\commands\chat.rs`
- Modify: `D:\code\WorkClaw\apps\runtime\src-tauri\src\commands\session_runs.rs`

**Step 1: Write the failing eval runner tests**

Add tests for non-network evaluation logic:

```rust
#[test]
fn evaluator_marks_warn_when_execution_succeeds_but_total_time_exceeds_pass_threshold() {
    let scenario = fixture_scenario();
    let mut report = EvalReport::passing("pm_weekly_summary");
    report.timing.total_duration_ms = 160_000;

    let evaluated = evaluate_report_against_scenario(report, &scenario);
    assert_eq!(evaluated.status, EvalReportStatus::Warn);
}

#[test]
fn evaluator_marks_fail_when_required_output_phrase_is_missing() {
    let scenario = fixture_scenario();
    let mut report = EvalReport::passing("pm_weekly_summary");
    report.final_output_excerpt = "只有统计，没有项目名称".to_string();

    let evaluated = evaluate_report_against_scenario(report, &scenario);
    assert_eq!(evaluated.status, EvalReportStatus::Fail);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml evaluator_marks -- --nocapture`

Expected: FAIL because the runner/evaluator modules do not exist.

**Step 3: Implement the CLI runner**

Implement `runner.rs` so it:
- loads `config.local.yaml` and the requested scenario YAML
- resolves `capability_id -> real workspace_root / entry_name / model profile`
- creates a dedicated run directory under `temp/agent-evals/<timestamp>__<scenario_id>/`
- initializes a local sqlite DB there using `init_db_at_dir`
- creates a session with `commands::chat::create_session_with_pool`
- inserts and runs the real user message through `SessionRuntime::run_send_message`
- reads run projections with `commands::session_runs::list_session_runs_with_pool`
- exports trace with `commands::session_runs::export_session_run_trace_with_pool`
- evaluates assertions and writes a structured report

Keep the CLI bin simple:

```rust
fn main() -> anyhow::Result<()> {
    let args = AgentEvalCliArgs::parse_from_env()?;
    let report = run_real_agent_eval(args)?;
    print_summary_and_exit(report);
}
```

Exit codes:
- `0` for `pass`
- `2` for `warn`
- `1` for `fail`

**Step 4: Run the focused Rust tests**

Run:
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml evaluator_marks -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml agent_evals -- --nocapture`

Expected: PASS for parser/evaluator/runner dry-path tests.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/evals/assertions.rs apps/runtime/src-tauri/src/agent/evals/runner.rs apps/runtime/src-tauri/src/bin/agent-evals.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/commands/session_runs.rs
git commit -m "feat(eval): add real-agent eval cli runner"
```

### Task 5: Add the Node wrapper, dry-run UX, and local manual verification lane

**Files:**
- Create: `D:\code\WorkClaw\scripts\run-agent-evals.mjs`
- Modify: `D:\code\WorkClaw\scripts\run-agent-evals.test.mjs`
- Modify: `D:\code\WorkClaw\package.json`
- Create: `D:\code\WorkClaw\agent-evals\README.md`

**Step 1: Write the failing wrapper tests**

Extend `scripts/run-agent-evals.test.mjs` with tests for:

```js
test("builds cargo command for real-agent eval runner", async () => {
  const mod = await import("./run-agent-evals.mjs");
  const cmd = mod.buildAgentEvalCommand({
    scenarioId: "pm_weekly_summary_xietao_2026_03_30_2026_04_04",
    projectRoot: "D:/code/WorkClaw",
  });

  assert.equal(cmd.command.includes("cargo"), true);
  assert.deepEqual(cmd.args.slice(-2), [
    "--scenario",
    "pm_weekly_summary_xietao_2026_03_30_2026_04_04",
  ]);
});
```

**Step 2: Run test to verify it fails**

Run: `node --test scripts/run-agent-evals.test.mjs`

Expected: FAIL because the wrapper does not exist yet.

**Step 3: Implement the Node wrapper and README**

Implement `scripts/run-agent-evals.mjs` so it:
- resolves repo root
- checks that `agent-evals/config/config.local.yaml` exists
- supports `--scenario <id>` and `--dry-run`
- shells out to:

```bash
cargo run --manifest-path apps/runtime/src-tauri/Cargo.toml --bin agent-evals -- --scenario <id>
```

- streams stdout/stderr
- prints the final report path

Add `agent-evals/README.md` with safe, non-secret guidance:
- what is committed vs local-only
- how to create `config.local.yaml`
- how to run the first scenario
- where reports land

**Step 4: Run wrapper tests and dry-run**

Run:
- `node --test scripts/run-agent-evals.test.mjs`
- `pnpm eval:agent-real --scenario pm_weekly_summary_xietao_2026_03_30_2026_04_04 --dry-run`

Expected:
- Node tests PASS
- dry-run prints the resolved cargo command, scenario path, local config path, and report directory without calling the model

**Step 5: Commit**

```bash
git add scripts/run-agent-evals.mjs scripts/run-agent-evals.test.mjs package.json agent-evals/README.md
git commit -m "feat(eval): add local manual real-agent eval entrypoint"
```

### Task 6: Execute the first live golden case and lock in the regression report format

**Files:**
- Modify: `D:\code\WorkClaw\agent-evals\README.md`
- Local-only: `D:\code\WorkClaw\agent-evals\config\config.local.yaml` (do not commit)
- Local-only output: `D:\code\WorkClaw\temp\agent-evals\...` (do not commit)

**Step 1: Prepare the local-only config**

Create local-only `agent-evals/config/config.local.yaml` with the real mappings:

```yaml
runtime:
  workspace_root: D:\code\WorkClaw
  cargo_manifest_path: apps/runtime/src-tauri/Cargo.toml

models:
  default_profile: real_eval_openai

providers:
  real_eval_openai:
    provider: openai
    model: gpt-5.4
    api_key_env: OPENAI_API_KEY

artifacts:
  output_dir: D:\code\WorkClaw\temp\agent-evals

capabilities:
  pm_weekly_summary:
    workspace_root: E:\code\work\飞书多维表格自动化skill
    entry_kind: workspace_skill
    entry_name: feishu-pm-hub

diagnostics:
  export_journal: true
  export_trace: true
  export_stdout_stderr: true
```

**Step 2: Run the real scenario**

Run:

```bash
pnpm eval:agent-real --scenario pm_weekly_summary_xietao_2026_03_30_2026_04_04
```

Expected:
- real model call happens
- real `feishu-pm` capability runs
- report is written under `temp/agent-evals/...`

**Step 3: Verify the report contents**

Inspect the produced report and verify:
- `status` is `pass` or `warn`
- `selected_skill` / `selected_runner` are populated
- total duration, route duration, and turn/tool counts are recorded
- the excerpted output includes the expected project names
- `journal_path` and `trace_path` are present

**Step 4: Update README with one concrete sample output block**

Add a short non-secret example showing:
- command used
- report path pattern
- meaning of `pass / warn / fail`

**Step 5: Commit**

```bash
git add agent-evals/README.md
git commit -m "docs(eval): document local real-agent golden case workflow"
```

### Task 7: Run the release-quality verification set

**Files:**
- No new files

**Step 1: Run focused Node tests**

Run: `node --test scripts/run-agent-evals.test.mjs`

Expected: PASS.

**Step 2: Run focused Rust tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml agent_evals session_runtime -- --nocapture`

Expected: PASS.

**Step 3: Run the existing fast Rust regression lane**

Run: `pnpm test:rust-fast`

Expected: PASS with no regressions in the existing runtime fast suite.

**Step 4: Run one real manual eval**

Run: `pnpm eval:agent-real --scenario pm_weekly_summary_xietao_2026_03_30_2026_04_04`

Expected:
- `pass` or `warn`
- report written to `temp/agent-evals/...`
- no secrets written into git-tracked files

**Step 5: Commit**

```bash
git add -A
git commit -m "test(eval): verify real-agent eval harness end to end"
```
