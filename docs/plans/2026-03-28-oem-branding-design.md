# OEM Branding Design

## Summary

WorkClaw should support OEM-style branding at build time so a maintainer can produce a differently named desktop app, such as `XXClaw`, by selecting a brand manifest and replacing a small set of source assets. The preferred implementation is a build-time branding pipeline rather than runtime switching.

This design keeps internal package and crate names stable while allowing user-visible brand surfaces, bundle identifiers, icons, and installer assets to vary per brand.

## Goals

- Build different branded installers from one codebase.
- Change visible product naming through one brand manifest.
- Replace packaged logos and icons through one brand asset set.
- Keep the default WorkClaw brand as the baseline brand.
- Minimize changes to runtime logic and avoid runtime brand switching complexity.

## Non-Goals

- Runtime brand switching after installation.
- Renaming internal Rust crates, npm workspace package names, or module folders.
- Automatically rewriting every historical prompt or vendor-compat string in V1.
- Supporting arbitrary downstream theme overrides unrelated to OEM packaging.

## Recommended Approach

Use a single source of truth per brand and materialize generated build inputs before packaging.

The preferred flow is:

1. Choose a brand key such as `workclaw` or `xxclaw`.
2. Load `branding/brands/<brand>/manifest.json`.
3. Validate the manifest and required assets.
4. Generate the frontend brand constants file.
5. Generate the Tauri config file consumed by packaging.
6. Copy or derive icon and installer assets into the expected Tauri asset paths.
7. Run the existing desktop build.

This keeps brand choice in the build pipeline, where Tauri product naming, identifiers, installer resources, and executable metadata already belong.

## Why Build-Time Branding

Build-time branding is the smallest safe path because the current repo already binds brand-sensitive values into Tauri packaging inputs:

- `apps/runtime/src-tauri/tauri.conf.json` contains `productName`, `identifier`, window title, bundle icons, and installer image paths.
- `apps/runtime/src-tauri/icons/` contains packaged icons and installer images.
- `apps/runtime/src/assets/branding/` contains frontend brand assets.
- Frontend state keys currently include a `workclaw:` prefix in `apps/runtime/src/App.tsx` and related tests.

Trying to switch these at runtime would create unnecessary complexity around:

- install and upgrade identity
- Windows app registration
- local persistence boundaries
- test expectations
- code paths that assume branding is fixed at build time

## Target Architecture

### 1. Brand Manifest

Add one manifest per brand:

`branding/brands/<brand>/manifest.json`

Suggested fields:

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

V1 should treat the following as required:

- `brandKey`
- `productName`
- `desktopTitle`
- `bundleIdentifier`
- `exeName`
- `localStoragePrefix`
- `assetsDir`

### 2. Brand Asset Set

Each brand owns a self-contained asset folder:

`branding/brands/<brand>/assets/`

Suggested layout:

- `logo/app-logo.png`
- `logo/app-icon-source.png` or `logo/app-icon-source.svg`
- `installer/nsis-header.bmp`
- `installer/nsis-sidebar.bmp`
- `installer/wix-banner.bmp`
- `installer/wix-dialog.bmp`

If later needed, the layout can expand to tray, splash, or updater assets without changing the manifest contract.

### 3. Generated Outputs

The branding pipeline should generate or refresh only a small set of controlled outputs:

- `apps/runtime/src/branding.generated.ts`
- `apps/runtime/src-tauri/tauri.generated.conf.json`
- `apps/runtime/src/assets/branding/current/*`
- `apps/runtime/src-tauri/icons/*`

The generated files should be treated as build inputs, not hand-edited sources.

## Build Pipeline

## Command Shape

Add a brand application script:

`node scripts/apply-brand.mjs --brand xxclaw`

Then route branded packaging through:

`pnpm build:runtime -- --brand xxclaw`

Suggested responsibilities for `scripts/apply-brand.mjs`:

- parse the requested brand key
- load the manifest
- validate required fields
- validate required source assets
- generate frontend branding constants
- generate Tauri config from a template
- copy or derive icon and installer assets into Tauri paths
- fail early with actionable errors when branding inputs are incomplete

## Tauri Config Strategy

Do not hand-maintain multiple Tauri configs.

Instead, keep a template file such as:

- `apps/runtime/src-tauri/tauri.conf.template.json`

and generate:

- `apps/runtime/src-tauri/tauri.generated.conf.json`

Then update the build invocation to point Tauri at the generated config.

This avoids drift between brands and keeps the WorkClaw defaults visible in one place.

## Frontend Branding Strategy

Frontend code should stop hardcoding visible brand strings and storage prefixes. Replace direct literals with imports from:

- `apps/runtime/src/branding.generated.ts`

V1 should use the generated frontend module for:

- app titlebar label
- welcome or setup branding
- frontend logo path
- localStorage prefix helpers

This is especially relevant because the repo currently hardcodes keys like:

- `workclaw:model-setup-hint-dismissed`
- `workclaw:initial-model-setup-completed`

Those should become helper-generated keys based on `localStoragePrefix`.

## Stable vs Variable Names

To keep the implementation safe, separate stable internal names from OEM-visible brand names.

Values that should remain stable in V1:

- Rust crate names
- npm workspace package names
- most source directory names
- internal module ids that are not user-visible

Values that should be OEM-variable in V1:

- app product name
- window title
- bundle identifier
- executable/display name
- frontend logo
- Tauri bundle icons
- NSIS and WiX branding images
- localStorage key prefix

Values that need explicit review before making OEM-variable:

- on-disk app data directory names
- SQLite database path names
- custom protocol handlers
- deep-link scheme names
- updater channels
- prompt and vendor-compat strings that mention WorkClaw or OpenClaw

## Compatibility Risks

### App Identity

Changing `bundleIdentifier` changes installation identity and upgrade behavior. This is useful for parallel OEM installs, but it means the OEM build should be treated as a different desktop app rather than an in-place rename of WorkClaw.

### Frontend Persistence

Changing `localStoragePrefix` means the frontend will not automatically reuse WorkClaw browser storage state. This is acceptable and desirable for separate branded builds.

### Data Directory and SQLite

If any runtime app-data path currently depends on branding, that path must either:

- stay stable in V1, or
- change in a backward-compatible, explicitly reviewed way

V1 should prefer keeping runtime data path rules unchanged unless OEM isolation is a stated requirement.

### Tests

Any tests asserting `WorkClaw`, `workclaw:`, or fixed installer asset paths will need controlled updates so they read the generated brand config or verify the current default brand.

## Proposed File Changes

Expected new files:

- `branding/brands/workclaw/manifest.json`
- `branding/brands/workclaw/assets/...`
- `scripts/apply-brand.mjs`
- `apps/runtime/src/branding.generated.ts`
- `apps/runtime/src-tauri/tauri.conf.template.json`

Expected modified files:

- `package.json`
- `apps/runtime/package.json`
- `scripts/start-runtime-dev.mjs`
- `scripts/check-installer-branding.test.mjs`
- `apps/runtime/src/App.tsx`
- frontend tests that assert hardcoded brand labels or localStorage keys

Depending on implementation detail, the existing `apps/runtime/src-tauri/tauri.conf.json` may become either:

- a generated file for the active brand, or
- a default-brand checked-in output derived from the template

The second option is safer for local developer ergonomics.

## Verification Strategy

Because this change affects packaging and installer branding, the verification set must include both normal code checks and release-sensitive checks.

Minimum expected verification after implementation:

- `pnpm test:installer`
- `pnpm build:runtime`

Likely additional checks if frontend constants or scripts change:

- `pnpm --dir apps/runtime test`
- `pnpm test:release`

If release docs or version metadata are touched, also run:

- `pnpm test:release-docs`
- `pnpm release:check-version`

## Release Readiness

This work is release-sensitive because it changes installer and packaging behavior. The branch should not be called ship-ready until installer and packaging checks pass against the new branding flow.

Current expected release verdict before implementation:

- `YELLOW`

Reason:

- the design is sound, but the build and verification pipeline has not yet been updated or exercised with an OEM brand.

## Rollout Plan

V1 should land in this order:

1. Introduce the manifest and asset directory structure with `workclaw` as the default brand.
2. Add the branding apply script and Tauri template flow.
3. Move frontend visible branding and storage key prefixes behind generated constants.
4. Update installer branding tests to validate the active generated config.
5. Add one sample OEM brand such as `xxclaw` to prove the path end to end.

## Recommendation

The smallest safe implementation path is:

- build-time brand selection only
- single manifest per brand
- generated frontend constants
- generated Tauri config from a template
- controlled icon and installer asset materialization
- no runtime switching
- no internal package renames

This gives WorkClaw a clean OEM story without destabilizing the runtime or overfitting the codebase to branding concerns.
