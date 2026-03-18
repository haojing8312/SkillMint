# OpenClaw-Style Tool Results Design

## Context

WorkClaw's built-in tools currently return a mix of:

- plain text intended for humans
- ad-hoc JSON strings
- error messages that describe failure but not the next recovery step

This creates three recurring problems:

1. The agent must often re-parse tool output from prose instead of consuming stable fields.
2. Follow-up tool calls are less reliable because paths, line numbers, match sets, and command outcomes are not always preserved in machine-friendly form.
3. Runtime guardrails and UI layers cannot consistently reason about progress, retries, or failure recovery because tool outcomes do not share one result contract.

Recent fixes around `list_dir` and repeated-loop detection reduced one concrete failure mode, but they also highlighted that the broader issue is protocol inconsistency across high-frequency tools.

The goal of this design is to move WorkClaw toward an OpenClaw-style tool-result model:

- structured results first
- concise summary text for humans and models
- explicit recovery hints on failure
- stable machine-readable details for downstream reasoning, UI display, and run-guard logic

## Goals

- Define one shared output pattern for high-frequency built-in tools.
- Migrate the highest-leverage tools first: `read_file`, `grep`, and `bash`.
- Preserve compatibility during migration by keeping readable summary text.
- Make future file and command tools follow the same result shape.
- Create a foundation for better loop detection, UI tool cards, and failure recovery.

## Non-Goals

- Full one-shot conversion of every built-in tool in a single patch.
- Replacing the current tool interface with a new Rust trait right away.
- Rebuilding the chat UI tool-event pipeline in the first phase.
- Introducing desktop-organizer-specific tools.

## Current Pain Points

### `read_file`

`read_file` currently returns only raw file text.

Problems:

- no stable `path` echo
- no `line_count`
- no `truncated` indicator
- no `encoding` or `size_bytes`
- difficult for later steps to distinguish file content from tool wrapper content

### `grep`

`grep` currently returns formatted prose-like output.

Problems:

- matches are embedded in text instead of a list of result objects
- no stable `total_matches`/`files_searched` fields
- difficult for the model to reuse exact match positions
- poor support for UI rendering or follow-up editing flows

### `bash`

`bash` currently returns formatted stdout/stderr text.

Problems:

- no stable `exit_code`
- timeout and failure are conveyed in prose
- background vs foreground outcomes use different semantics
- harder to classify no-progress vs meaningful state changes

### Cross-tool issues

- inconsistent success shapes
- inconsistent error shapes
- no shared recovery guidance
- mixed text/JSON conventions with no documented contract

## Reference Direction From OpenClaw

OpenClaw's runtime leans on structured tool parameters and structured outcomes rather than purely human-readable prose. The important ideas to carry over are:

1. Preserve exact machine-consumable values for the next step.
2. Separate human-readable summaries from structured details.
3. Normalize failure reporting so runtime logic can classify and recover consistently.
4. Keep the protocol generic instead of baking domain-specific workflow assumptions into tools.

WorkClaw does not need to clone OpenClaw's code or exact types. It should adopt the same architectural direction while fitting the existing Rust + Tauri runtime.

## Proposed Result Contract

High-frequency tools should move toward a shared serialized result pattern:

```json
{
  "ok": true,
  "tool": "read_file",
  "summary": "Read 182 lines from docs/spec.md",
  "details": {
    "...": "tool-specific structured fields"
  }
}
```

Error form:

```json
{
  "ok": false,
  "tool": "read_file",
  "summary": "Could not read docs/spec.md",
  "error_code": "FILE_NOT_FOUND",
  "error_message": "The requested path does not exist.",
  "recovery_hint": "Use list_dir or file_stat to verify the exact path before retrying.",
  "details": {
    "path": "docs/spec.md"
  }
}
```

### Contract rules

- `summary` is short, human-readable, and suitable for UI display.
- `details` contains the exact machine-usable fields.
- `error_code` must be stable and predictable for common failures.
- `recovery_hint` should tell the agent what tool or action to try next.
- Tools may still expose the most important human-readable information inside `summary`, but downstream logic should prefer `details`.

## Migration Strategy

### Phase 1: Shared pattern without changing the tool trait

Keep the current `Tool::execute(...) -> Result<String>` trait.

Within that constraint:

- success results return serialized JSON strings with `ok`, `summary`, and `details`
- failures continue to raise Rust errors, but error text becomes more structured and recovery-oriented
- executor and UI stay compatible because they already transport strings

This phase gives us most of the benefit without a runtime-wide trait rewrite.

### Phase 2: Executor and UI become structure-aware

Add helper parsing in executor/UI layers so:

- tool cards can show `summary`
- machine logic can use `details`
- run guard and future orchestration logic can hash normalized structured outcomes instead of freeform text

### Phase 3: Expand to the rest of the toolset

After validating the pattern on `read_file`, `grep`, and `bash`, migrate:

- `file_copy`
- `file_move`
- `file_delete`
- `file_stat`
- `list_dir`
- `edit`

## Proposed Phase 1 Tool Shapes

### `read_file`

Success:

```json
{
  "ok": true,
  "tool": "read_file",
  "summary": "Read file docs/spec.md",
  "details": {
    "path": "docs/spec.md",
    "absolute_path": "D:/code/WorkClaw/docs/spec.md",
    "size_bytes": 4210,
    "line_count": 182,
    "content": "...",
    "truncated": false
  }
}
```

### `grep`

Success:

```json
{
  "ok": true,
  "tool": "grep",
  "summary": "Found 7 matches across 3 files",
  "details": {
    "pattern": "loop_detected",
    "searched_path": "apps/runtime/src-tauri/src",
    "files_searched": 14,
    "total_matches": 7,
    "truncated": false,
    "matches": [
      {
        "path": "agent/run_guard.rs",
        "line": 88,
        "text": "RunStopReasonKind::LoopDetected => \"loop_detected\","
      }
    ]
  }
}
```

### `bash`

Success or failure:

```json
{
  "ok": true,
  "tool": "bash",
  "summary": "Command completed with exit code 0",
  "details": {
    "command": "git status --short",
    "exit_code": 0,
    "timed_out": false,
    "stdout": "...",
    "stderr": "",
    "background": false
  }
}
```

Background launch:

```json
{
  "ok": true,
  "tool": "bash",
  "summary": "Started background process 12",
  "details": {
    "command": "pnpm app",
    "background": true,
    "process_id": 12
  }
}
```

## Compatibility Plan

- Keep `Tool::execute` string-based in phase 1.
- Keep summary information present so tool-call transcripts remain readable.
- Update tests to assert on parsed JSON payloads rather than only string fragments.
- Do not change every tool at once.

## Risks

### Risk: existing prompts or tests assume raw text-only outputs

Mitigation:

- migrate only the highest-priority tools first
- update prompt guidance and targeted tests together
- retain concise `summary` text in every result

### Risk: UI tool cards may show raw JSON blobs

Mitigation:

- accept temporary raw JSON in phase 1 where necessary
- add executor/UI parsing in the next batch

### Risk: run guard hashes may change

Mitigation:

- explicitly use normalized structured output once available
- verify repeated-success and repeated-failure guard tests

## Testing Strategy

Phase 1 should include:

- dedicated unit tests for each migrated tool
- prompt assembly tests where tool guidance changes
- regression coverage for loop detection and repeated failures
- `pnpm test:rust-fast`

## Recommendation

Implement the OpenClaw-style migration in batches, starting with `read_file`, `grep`, and `bash`, while keeping the current trait and transport shape. This is the smallest safe path that still counts as a real architectural move instead of another one-off patch.
