# Branding Layout

WorkClaw supports build-time OEM branding through one brand manifest and one asset directory per brand.

## Directory Layout

```text
branding/
  brands/
    workclaw/
      manifest.json
      assets/
        logo/
        installer/
```

## Manifest Contract

Each brand manifest must provide the values the build pipeline needs to generate app identity and packaging inputs:

- `brandKey`: stable brand key used on the command line
- `productName`: user-visible app name
- `desktopTitle`: title shown in the desktop window
- `bundleIdentifier`: Tauri bundle identifier
- `publisher`: publisher or company label for packaging
- `exeName`: executable display name
- `localStoragePrefix`: prefix for frontend storage keys
- `protocolScheme`: custom URI scheme name, if used
- `assetsDir`: relative path to the brand asset directory, resolved from the manifest directory

## Asset Contract

Each brand should provide assets under `branding/brands/<brand>/assets/`.

Expected V1 asset groups:

- `logo/`: frontend logo assets. The build currently reads `logo/app-logo.png`.
- `installer/`: installer-specific branding images such as NSIS and WiX artwork. These are copied into `apps/runtime/src-tauri/icons/installer/`.

The build pipeline treats these assets as source inputs and materializes the packaged copies into the runtime and Tauri asset paths before build time.

Current materialized targets:

- `logo/app-logo.png` -> `apps/runtime/src/assets/branding/current/app-logo.png`
- `logo/app-logo.png` -> generated Tauri icon tree in `apps/runtime/src-tauri/icons/**/*` via `pnpm --dir apps/runtime tauri icon`
- `installer/*` -> `apps/runtime/src-tauri/icons/installer/*`

## Default Brand

`workclaw` is the baseline brand and should match the current WorkClaw identity until an alternate brand is selected.

## Sample OEM Brand

`xxclaw` is included as a sample downstream brand to prove the OEM pipeline end to end. Its asset folders are tracked with placeholder files so the build scripts can validate the expected directory layout before real artwork is added.

At the moment the sample brand reuses copied placeholder artwork from the baseline WorkClaw assets so the OEM copy pipeline can be exercised without blocking on final design files.
