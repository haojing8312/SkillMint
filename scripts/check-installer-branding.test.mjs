import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import path from "node:path";

const projectRoot = process.cwd();
const tauriConfigPath = path.join(
  projectRoot,
  "apps",
  "runtime",
  "src-tauri",
  "tauri.conf.json",
);

function readConfig() {
  return JSON.parse(readFileSync(tauriConfigPath, "utf8"));
}

function resolveAsset(relativePath) {
  return path.join(projectRoot, "apps", "runtime", "src-tauri", relativePath);
}

test("windows installers are branded and localized for zh-CN", () => {
  const config = readConfig();
  const windows = config?.bundle?.windows;
  const nsis = windows?.nsis;
  const wix = windows?.wix;
  const resources = config?.bundle?.resources;

  assert.ok(nsis, "Expected bundle.windows.nsis to be configured");
  assert.equal(nsis.installerIcon, "icons/icon.ico");
  assert.deepEqual(nsis.languages, ["SimpChinese"]);
  assert.equal(nsis.displayLanguageSelector, false);
  assert.equal(nsis.headerImage, "icons/installer/nsis-header.bmp");
  assert.equal(nsis.sidebarImage, "icons/installer/nsis-sidebar.bmp");

  assert.ok(wix, "Expected bundle.windows.wix to be configured");
  assert.equal(wix.language, "zh-CN");
  assert.equal(wix.bannerPath, "icons/installer/wix-banner.bmp");
  assert.equal(wix.dialogImagePath, "icons/installer/wix-dialog.bmp");

  assert.ok(Array.isArray(resources), "Expected bundle.resources to be configured");
  assert.match(
    JSON.stringify(resources),
    /sidecar-runtime/i,
    "Expected bundle.resources to include packaged sidecar runtime assets",
  );

  for (const assetPath of [
    nsis.installerIcon,
    nsis.headerImage,
    nsis.sidebarImage,
    wix.bannerPath,
    wix.dialogImagePath,
  ]) {
    assert.ok(existsSync(resolveAsset(assetPath)), `Expected installer asset to exist: ${assetPath}`);
  }
});
