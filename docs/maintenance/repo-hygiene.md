# Repo Hygiene

## Why This Exists

WorkClaw uses long-running AI-assisted development across runtime, sidecar, Rust, skills, and docs. Repo hygiene review exists to stop temporary artifacts, dead code, duplicated implementations, and stale references from silently becoming part of the long-term source of truth.

## Finding Categories

- temporary-artifacts
- orphan-files
- dead-code
- duplicate-implementations
- oversized-file
- import-cycle
- stale-docs
- stale-skill-or-command-references
- generated-or-runtime-owned-candidates

## Confidence Levels

- confirmed: multiple signals or direct evidence show the candidate is safe to remove or replace.
- likely: the candidate looks removable, but one more check is needed before deletion.
- uncertain: the candidate needs owner knowledge, runtime evidence, or compatibility review.
- high-risk: the candidate is generated, runtime-owned, dynamically discovered, or config-driven unless a rule explicitly marks it safe.

## Allowed Actions

- Report findings without deleting anything when the task is only review or triage.
- Group candidates into small cleanup batches after repo hygiene review.
- Delete only confirmed candidates in a reviewed cleanup batch.
- Prefer deprecation, relocation, or compatibility fallbacks when removal could affect runtime behavior or older worktrees.
- Keep suspicious files and code in place until they are classified.

## Default Workflow

1. Run `pnpm review:repo-hygiene`.
2. Use focused subcommands when you need one narrow signal only:
   - `pnpm review:repo-hygiene:deadcode`
   - `pnpm review:repo-hygiene:artifacts`
   - `pnpm review:repo-hygiene:drift`
   - `pnpm review:repo-hygiene:dup`
   - `pnpm review:repo-hygiene:loc`
   - `pnpm review:repo-hygiene:cycles`
3. For frontend large-file governance, run `pnpm report:frontend-large-files` alongside repo hygiene review when a React or TypeScript surface is growing.
4. Review the report and triage candidates by finding category and confidence.
5. Use `workclaw-repo-hygiene-review` to classify findings before any destructive edit.
6. Choose the smallest cleanup batch that is still well supported.
7. Use `workclaw-cleanup-execution` only for the reviewed batch.
8. Run `workclaw-change-verification` when the cleanup changes code, tests, docs, or skill files.

Reports are written to `.artifacts/repo-hygiene/` for local review and should stay untracked.
The GitHub Actions `Repo Hygiene` workflow also runs this command in a non-blocking lane and uploads the generated report as a workflow artifact for review.

## OpenClaw-Style Check Layers

- `dead-code`: static dead-code candidates for TypeScript and Rust.
- `duplicate-implementations`: duplicated code candidates from `jscpd`. These are review-first signals, not auto-delete instructions.
- `oversized-file`: governance triggers for runtime frontend and Tauri Rust files using the repo's documented line-count thresholds.
- `import-cycle`: circular import candidates for runtime TypeScript surfaces.
- `temporary-artifacts` and `stale-doc-or-skill-reference`: housekeeping and drift signals for long-running AI-assisted work.

This mirrors the OpenClaw pattern of small named checks under one umbrella command instead of relying on a single general-purpose linter.

## Oversized File Governance

Oversized-file findings are not an instruction to split mechanically. Use them as a trigger to check whether one file is mixing too many responsibilities.

For runtime frontend surfaces, prefer this order:

1. Extract pure display sections into `components/<domain>/...`.
2. Extract reusable stateful orchestration into `use*.ts` hooks.
3. Extract pure projections, labels, grouping, and formatting into helper modules.
4. Leave final page composition and small view-local glue in the root component.

Recent validated runtime pattern:

- Keep `ChatView.tsx` as the composition root.
- Move viewport and focus behavior to `components/chat/useChatViewportController.ts`.
- Move derived message windows and side-panel models to `components/chat/useChatDerivedViewModels.ts`.
- Move group-run and delegation projections to `components/chat/chatGroupRunHelpers.ts`.
- Move standalone JSX sections such as dialogs, banners, and install panels into focused `components/chat/*` files.

Recent validated settings pattern:

- Keep `useFeishuSettingsController.ts` as the orchestration root that owns effects, polling, and section assembly only.
- Move async load/save/install/start actions to `components/settings/feishu/feishuSettingsControllerActions.ts`.
- Move default object shapes to `components/settings/feishu/feishuSettingsControllerDefaults.ts`.
- Move pure section prop derivation to `components/settings/feishu/feishuSettingsControllerViewModel.ts`.
- Keep large settings sections as composition roots and split stable UI blocks into focused files such as:
  - `components/settings/feishu/FeishuEnvironmentPanel.tsx`
  - `components/settings/feishu/FeishuAuthorizationPanel.tsx`
  - `components/settings/feishu/FeishuAdvancedSettingsForm.tsx`
  - `components/settings/feishu/FeishuConnectionDetailsPanel.tsx`

This is the preferred cleanup direction for future oversized runtime frontend files. The goal is stable responsibility boundaries, not line-count gaming.

## High-Risk Surfaces

- Generated files and generated directories.
- Runtime-owned artifacts and state.
- Dynamically discovered files, commands, or skills.
- Config-driven outputs and paths.
- Legacy compatibility surfaces that may still be read by old worktrees or older data.
- Any candidate that appears unused in only one static signal.

## Repo-Local Skills

- `workclaw-repo-hygiene-review` - review and classify findings only.
- `workclaw-cleanup-execution` - execute only after a reviewed batch is selected.
