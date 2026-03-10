# Quick Setup Search Onboarding Design

**Goal:** Extend the existing quick model setup dialog into a two-step onboarding wizard so first-time users configure both a model and a search engine before completing the initial setup experience.

## Problem

The current first-use and developer-triggered quick setup flow only configures a model. Search configuration exists separately in Settings, so users can finish onboarding without web search capability even though agent workflows depend on search for good results.

## Decision

Use a single modal wizard with two steps:

1. `模型配置`
2. `搜索引擎`

After the model step is saved successfully, the dialog advances automatically to the search step.

## Behavior

- First launch:
  - Opening quick setup shows step 1.
  - Saving the model advances to step 2 instead of closing.
  - The dialog remains blocking until at least one model and one search config both exist.
  - Escape, overlay click, and cancel remain disabled until both steps are complete.
- Opened manually from Settings:
  - Opening quick setup still starts at step 1.
  - Saving the model advances to step 2.
  - Step 2 includes a skip/close path because this entry is non-blocking.
- Completion state:
  - Initial onboarding is considered complete only after `models.length > 0 && searchConfigs.length > 0`.
  - The persisted first-use completion marker is written only after both are present.

## Implementation Shape

- Keep the wizard shell in `apps/runtime/src/App.tsx`.
- Extract reusable search configuration UI into a shared component used by:
  - `apps/runtime/src/App.tsx` quick setup wizard step 2
  - `apps/runtime/src/components/SettingsView.tsx`
- Share search presets and helper functions through a small module instead of duplicating logic.

## Testing

- Update App tests to cover:
  - saving model advances to search step instead of closing
  - first-launch flow only completes after saving a search config
  - settings-triggered quick setup can skip the search step
- Update SettingsView tests only as needed to keep the reused search form rendering stable.
