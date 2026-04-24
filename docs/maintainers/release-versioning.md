# Release Versioning and Tags

This project uses SemVer release tags (`vX.Y.Z`) and enforces version consistency across:

- `apps/runtime/package.json`
- `apps/runtime/src-tauri/tauri.conf.json`
- `apps/runtime/src-tauri/Cargo.toml`

CI will fail release builds if these versions do not match the pushed tag. Tag pushes trigger `.github/workflows/release-desktop.yml`, which builds Windows x64 `setup.exe`, Linux x64 `amd64.deb`, and Linux arm64 `arm64.deb` packages.

## Recommended Workflow

1. Update all three runtime versions in one commit.
2. Run local checks:
   - `pnpm test:release`
   - `pnpm release:check-version vX.Y.Z`
3. Create annotated tag:
   - `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
4. Push commit and tag:
   - `git push origin main`
   - `git push origin vX.Y.Z`

## Failure Handling

If release CI fails because of version mismatch:

1. Fix version files in a new commit.
2. Prefer a new patch tag (for example, `v0.2.1`) instead of rewriting an existing tag.
3. Push the new tag to trigger a fresh release workflow.

## SemVer Rules

- Patch (`X.Y.Z+1`): bugfixes, no breaking changes.
- Minor (`X.Y+1.0`): new backward-compatible features.
- Major (`X+1.0.0`): breaking changes.
