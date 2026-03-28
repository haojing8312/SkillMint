# OEM Branding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add build-time OEM branding so WorkClaw can produce differently branded desktop installers from one codebase by selecting a brand manifest and brand asset set.

**Architecture:** Introduce a brand manifest plus asset directory per brand, generate a frontend branding module and a Tauri config from templates, and route build scripts through a single brand-apply step before packaging. Keep internal package and crate names stable while moving user-visible branding and installer assets behind generated inputs.

**Tech Stack:** Node.js ESM scripts, pnpm workspace scripts, Tauri 2 desktop packaging, React + TypeScript frontend, Windows NSIS/WiX installer assets

---

### Task 1: Establish the branding source-of-truth layout

**Files:**
- Create: `branding/brands/workclaw/manifest.json`
- Create: `branding/brands/workclaw/assets/logo/.gitkeep`
- Create: `branding/brands/workclaw/assets/installer/.gitkeep`
- Create: `branding/README.md`
- Reference: `docs/plans/2026-03-28-oem-branding-design.md`

**Step 1: Create the manifest with the default WorkClaw brand**

Create `branding/brands/workclaw/manifest.json` with the baseline values copied from the current checked-in defaults:

```json
{
  "brandKey": "workclaw",
  "productName": "WorkClaw",
  "desktopTitle": "WorkClaw",
  "bundleIdentifier": "dev.workclaw.runtime",
  "publisher": "WorkClaw",
  "exeName": "WorkClaw",
  "localStoragePrefix": "workclaw",
  "protocolScheme": "workclaw",
  "assetsDir": "./assets"
}
```

**Step 2: Add the brand assets folder contract**

Create empty placeholder folders so future contributors know the expected structure:

- `branding/brands/workclaw/assets/logo/`
- `branding/brands/workclaw/assets/installer/`

**Step 3: Document the expected source asset contract**

Create `branding/README.md` describing:

- one folder per brand
- the manifest schema
- the expected asset filenames
- how the build scripts consume the assets

**Step 4: Verify the new structure exists**

Run: `Get-ChildItem branding/brands/workclaw -Recurse`

Expected: manifest plus `assets/logo` and `assets/installer` appear

**Step 5: Commit**

```bash
git add branding/ docs/plans/2026-03-28-oem-branding-design.md
git commit -m "chore: add oem branding source layout"
```

### Task 2: Add the brand application script and generated output contract

**Files:**
- Create: `scripts/apply-brand.mjs`
- Create: `apps/runtime/src/branding.generated.ts`
- Create: `apps/runtime/src-tauri/tauri.conf.template.json`
- Modify: `apps/runtime/src-tauri/tauri.conf.json`
- Modify: `package.json`
- Modify: `apps/runtime/package.json`

**Step 1: Write the failing script test or assertion harness**

If there is no existing script test harness for this flow, add a small Node test file:

- `scripts/apply-brand.test.mjs`

The first failing test should:

- load the default WorkClaw brand
- run the brand script against a temp output area or the repo outputs
- assert that `branding.generated.ts` contains `productName: "WorkClaw"`
- assert that generated Tauri config contains `productName`, `identifier`, and installer image paths

Example test shape:

```js
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("apply-brand generates frontend and tauri branding outputs for workclaw", async () => {
  // invoke script
  // read outputs
  assert.match(readFileSync(frontendOutput, "utf8"), /WorkClaw/);
  assert.equal(config.productName, "WorkClaw");
});
```

**Step 2: Run the test to verify it fails**

Run: `node --test scripts/apply-brand.test.mjs`

Expected: FAIL because the script and outputs do not exist yet

**Step 3: Implement the brand application script**

Implement `scripts/apply-brand.mjs` with these responsibilities:

- parse `--brand <key>`
- default to `workclaw` when omitted
- load `branding/brands/<key>/manifest.json`
- validate required manifest fields
- resolve the brand assets directory
- generate `apps/runtime/src/branding.generated.ts`
- generate `apps/runtime/src-tauri/tauri.conf.json` from `apps/runtime/src-tauri/tauri.conf.template.json`
- copy brand assets into:
  - `apps/runtime/src/assets/branding/current/`
  - `apps/runtime/src-tauri/icons/installer/`
  - `apps/runtime/src-tauri/icons/` for the main app icons

Keep the script idempotent so repeated runs with the same brand are stable.

**Step 4: Add the template-based Tauri config**

Move the current brand-sensitive values in `apps/runtime/src-tauri/tauri.conf.json` into `apps/runtime/src-tauri/tauri.conf.template.json` using placeholders or a template object, for example:

```json
{
  "productName": "__PRODUCT_NAME__",
  "identifier": "__BUNDLE_IDENTIFIER__",
  "app": {
    "windows": [{ "title": "__DESKTOP_TITLE__" }]
  }
}
```

Then make `apps/runtime/src-tauri/tauri.conf.json` the generated default-brand output.

**Step 5: Wire the scripts**

Update `package.json` and `apps/runtime/package.json` so:

- build commands run `apply-brand` first
- dev commands apply the default WorkClaw brand before launch
- an explicit helper exists, for example:
  - `brand:apply`
  - `build:runtime:brand`

**Step 6: Run the targeted script test**

Run: `node --test scripts/apply-brand.test.mjs`

Expected: PASS

**Step 7: Commit**

```bash
git add scripts/apply-brand.mjs scripts/apply-brand.test.mjs apps/runtime/src/branding.generated.ts apps/runtime/src-tauri/tauri.conf.template.json apps/runtime/src-tauri/tauri.conf.json package.json apps/runtime/package.json
git commit -m "feat: add build-time brand generation pipeline"
```

### Task 3: Move frontend visible branding behind generated constants

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Create: `apps/runtime/src/lib/branding.ts`
- Modify: `apps/runtime/src/__tests__/App.window-chrome.test.tsx`
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`
- Modify: `apps/runtime/src/__tests__/App.sidebar-navigation-selected-session.test.tsx`

**Step 1: Write a failing frontend test for generated branding**

Add or update a test to verify:

- the desktop titlebar renders the generated product name
- localStorage keys are built from the generated prefix rather than hardcoded `workclaw:`

Example assertion shape:

```tsx
expect(screen.getByText(BRANDING.productName)).toBeInTheDocument();
expect(storageKey("initial-model-setup-completed")).toBe("workclaw:initial-model-setup-completed");
```

**Step 2: Run the targeted frontend tests**

Run: `pnpm --dir apps/runtime test -- App.window-chrome`

Expected: FAIL because code still hardcodes `WorkClaw`

**Step 3: Add a small branding helper**

Create `apps/runtime/src/lib/branding.ts` that wraps the generated file:

```ts
import { BRANDING } from "../branding.generated";

export function storageKey(name: string): string {
  return `${BRANDING.localStoragePrefix}:${name}`;
}
```

Keep this helper tiny and focused on frontend runtime use.

**Step 4: Replace hardcoded visible brand usage**

Update `apps/runtime/src/App.tsx` to consume generated branding for:

- titlebar product label
- frontend logo path if currently hardcoded
- localStorage key construction

Do not replace unrelated compatibility strings in this task.

**Step 5: Update the tests to use the helper and generated constants**

Adjust the listed test files so expectations come from the generated branding module or the helper instead of fixed string literals.

**Step 6: Run the frontend tests**

Run:

- `pnpm --dir apps/runtime test -- App.window-chrome`
- `pnpm --dir apps/runtime test -- App.model-setup-hint`

Expected: PASS

**Step 7: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/lib/branding.ts apps/runtime/src/__tests__/App.window-chrome.test.tsx apps/runtime/src/__tests__/App.model-setup-hint.test.tsx apps/runtime/src/__tests__/App.sidebar-navigation-selected-session.test.tsx
git commit -m "refactor: route frontend branding through generated config"
```

### Task 4: Update installer and packaging verification for generated branding

**Files:**
- Modify: `scripts/check-installer-branding.test.mjs`
- Modify: `scripts/start-runtime-dev.mjs`
- Modify: `package.json`

**Step 1: Write the failing verification expectation**

Extend `scripts/check-installer-branding.test.mjs` so it validates:

- the active Tauri config was generated from the brand manifest
- the configured installer asset paths exist after brand application
- the product name and identifier in the active config are non-empty and brand-derived

**Step 2: Run the installer verification**

Run: `pnpm test:installer`

Expected: FAIL if the script still assumes only fixed checked-in WorkClaw values

**Step 3: Update the dev launcher to ensure branding is applied**

Modify `scripts/start-runtime-dev.mjs` so local dev ensures the default brand is materialized before `pnpm --dir apps/runtime tauri dev` runs. Keep the default brand behavior transparent for existing contributors.

**Step 4: Update script routing**

Ensure the root scripts make it easy to run:

- default local dev with WorkClaw branding
- branded packaging through an explicit brand argument

**Step 5: Run verification**

Run:

- `pnpm test:installer`
- `pnpm test:release`

Expected: PASS

**Step 6: Commit**

```bash
git add scripts/check-installer-branding.test.mjs scripts/start-runtime-dev.mjs package.json
git commit -m "test: verify generated installer branding flow"
```

### Task 5: Prove the OEM path with a sample secondary brand

**Files:**
- Create: `branding/brands/xxclaw/manifest.json`
- Create: `branding/brands/xxclaw/assets/logo/.gitkeep`
- Create: `branding/brands/xxclaw/assets/installer/.gitkeep`
- Modify: `branding/README.md`

**Step 1: Add the sample OEM brand**

Create `branding/brands/xxclaw/manifest.json` with obviously distinct values:

```json
{
  "brandKey": "xxclaw",
  "productName": "XXClaw",
  "desktopTitle": "XXClaw",
  "bundleIdentifier": "dev.xxclaw.runtime",
  "publisher": "XXClaw",
  "exeName": "XXClaw",
  "localStoragePrefix": "xxclaw",
  "protocolScheme": "xxclaw",
  "assetsDir": "./assets"
}
```

**Step 2: Add placeholder assets or documented sample assets**

If no real OEM images are available yet, add placeholder files or keep `.gitkeep` plus explicit README notes so the script fails with a clear message when assets are missing.

**Step 3: Add a script-level proof test**

Extend `scripts/apply-brand.test.mjs` to invoke the script with `--brand xxclaw` and assert:

- generated frontend module contains `XXClaw`
- generated Tauri config uses `dev.xxclaw.runtime`
- localStorage prefix is `xxclaw`

**Step 4: Run the proof test**

Run: `node --test scripts/apply-brand.test.mjs`

Expected: PASS

**Step 5: Commit**

```bash
git add branding/brands/xxclaw branding/README.md scripts/apply-brand.test.mjs
git commit -m "test: prove oem branding with sample brand manifest"
```

### Task 6: Run the full verification set and document remaining limits

**Files:**
- Modify: `docs/plans/2026-03-28-oem-branding-design.md`
- Modify: `branding/README.md`

**Step 1: Run the minimum required verification**

Run:

- `pnpm --dir apps/runtime test`
- `pnpm test:installer`
- `pnpm test:release`
- `pnpm build:runtime`

Expected: PASS

**Step 2: If any packaging or release-doc surfaces changed, run the additional release checks**

Run if applicable:

- `pnpm release:check-version`
- `pnpm test:release-docs`

Expected: PASS

**Step 3: Document the known V1 boundaries**

Update docs to explicitly note that V1 does not yet OEM:

- runtime database directory naming
- protocol registration beyond manifest fields, unless implemented
- prompt and vendor-compat copy that still intentionally mentions WorkClaw or OpenClaw

**Step 4: Record the exact commands and outcomes**

Add the final verification commands and pass/fail state to the docs or implementation notes so release-sensitive evidence is easy to audit later.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-28-oem-branding-design.md branding/README.md
git commit -m "docs: record oem branding verification and v1 boundaries"
```

## Notes for the Implementer

- Keep `apps/runtime/src-tauri/tauri.conf.json` usable for local contributors. Default-brand generation should not make local dev fragile.
- Do not rename crates, package folders, or broad internal identifiers in this plan.
- Be selective about replacing `WorkClaw` strings. Only move user-visible or build-sensitive strings behind branding in V1.
- Prefer deterministic generation and clear failures over magical fallback behavior.
- If icon derivation from one source image becomes time-consuming, allow V1 to copy pre-rendered brand assets first and add derivation later.
