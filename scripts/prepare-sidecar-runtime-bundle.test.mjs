import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  buildDeployCommand,
  isRetryableWindowsDeployError,
  pruneNonRuntimeBundlePaths,
} from "./prepare-sidecar-runtime-bundle.mjs";

test("buildDeployCommand disables bin links via environment on Windows-safe deploys", () => {
  const runner = { command: "pnpm.cmd", args: [] };
  const baseEnv = { PATH: "C:\\bin" };

  const result = buildDeployCommand(runner, 10, "D:\\bundle", baseEnv);

  assert.equal(result.command, "pnpm.cmd");
  assert.deepEqual(result.args, [
    "--filter",
    "workclaw-runtime-sidecar",
    "deploy",
    "--prod",
    "--config.bin-links=false",
    "--legacy",
    "D:\\bundle",
  ]);
  assert.equal(result.env.npm_config_bin_links, "false");
  assert.equal(result.env.pnpm_config_bin_links, "false");
  assert.equal(result.env.NPM_CONFIG_BIN_LINKS, "false");
  assert.equal(result.env.PNPM_CONFIG_BIN_LINKS, "false");
  assert.equal(result.env.PATH, "C:\\bin");
});

test("buildDeployCommand omits legacy flag for older pnpm versions", () => {
  const runner = { command: "pnpm", args: ["--dir", "apps/runtime/sidecar"] };

  const result = buildDeployCommand(runner, 9, "/tmp/bundle", {});

  assert.deepEqual(result.args, [
    "--dir",
    "apps/runtime/sidecar",
    "--filter",
    "workclaw-runtime-sidecar",
    "deploy",
    "--prod",
    "--config.bin-links=false",
    "/tmp/bundle",
  ]);
  assert.equal(result.env.npm_config_bin_links, "false");
  assert.equal(result.env.pnpm_config_bin_links, "false");
});

test("isRetryableWindowsDeployError recognizes transient playwright bin failures on Windows", () => {
  assert.equal(
    isRetryableWindowsDeployError(
      "WARN Failed to create bin at D:\\bundle\\node_modules\\.bin\\playwright. ENOENT: no such file or directory, chmod 'D:\\bundle\\node_modules\\.bin\\playwright.ps1'\nEPERM: operation not permitted, open 'D:\\bundle\\node_modules\\.bin\\playwright.CMD'",
      "win32",
    ),
    true,
  );
});

test("isRetryableWindowsDeployError ignores unrelated failures", () => {
  assert.equal(isRetryableWindowsDeployError("ERR_PNPM_FETCH_404", "win32"), false);
});

test("pruneNonRuntimeBundlePaths removes bundled MCP SDK example trees without touching runtime files", (t) => {
  const bundleDir = mkdtempSync(path.join(os.tmpdir(), "sidecar-runtime-bundle-"));
  t.after(() => rmSync(bundleDir, { recursive: true, force: true }));

  const pnpmExamplesDir = path.join(
    bundleDir,
    "node_modules",
    ".pnpm",
    "@modelcontextprotocol+sdk@1.27.1_zod@4.3.6",
    "node_modules",
    "@modelcontextprotocol",
    "sdk",
    "dist",
    "cjs",
    "examples",
  );
  const directExamplesDir = path.join(
    bundleDir,
    "node_modules",
    "@modelcontextprotocol",
    "sdk",
    "dist",
    "esm",
    "examples",
  );
  const hoistedExamplesDir = path.join(
    bundleDir,
    "node_modules",
    ".pnpm",
    "node_modules",
    "workclaw-runtime-sidecar",
    "node_modules",
    "@modelcontextprotocol",
    "sdk",
    "dist",
    "cjs",
    "examples",
  );
  const runtimeEntry = path.join(
    bundleDir,
    "node_modules",
    ".pnpm",
    "@modelcontextprotocol+sdk@1.27.1_zod@4.3.6",
    "node_modules",
    "@modelcontextprotocol",
    "sdk",
    "dist",
    "cjs",
    "server",
    "index.js",
  );

  mkdirSync(pnpmExamplesDir, { recursive: true });
  writeFileSync(path.join(pnpmExamplesDir, "simpleOAuthClientProvider.js"), "export {};\n");
  mkdirSync(directExamplesDir, { recursive: true });
  writeFileSync(path.join(directExamplesDir, "serverWithTools.js"), "export {};\n");
  mkdirSync(hoistedExamplesDir, { recursive: true });
  writeFileSync(path.join(hoistedExamplesDir, "streamableHttpWithSseFallbackClient.js"), "export {};\n");
  mkdirSync(path.dirname(runtimeEntry), { recursive: true });
  writeFileSync(runtimeEntry, "module.exports = {};\n");

  const prunedPaths = pruneNonRuntimeBundlePaths(bundleDir);

  assert.deepEqual(prunedPaths, [directExamplesDir, hoistedExamplesDir, pnpmExamplesDir].sort());
  assert.equal(existsSync(directExamplesDir), false);
  assert.equal(existsSync(hoistedExamplesDir), false);
  assert.equal(existsSync(pnpmExamplesDir), false);
  assert.equal(existsSync(runtimeEntry), true);
});

test("pruneNonRuntimeBundlePaths is a no-op when no targeted example directories exist", (t) => {
  const bundleDir = mkdtempSync(path.join(os.tmpdir(), "sidecar-runtime-bundle-empty-"));
  t.after(() => rmSync(bundleDir, { recursive: true, force: true }));

  mkdirSync(path.join(bundleDir, "node_modules", ".pnpm"), { recursive: true });

  assert.deepEqual(pruneNonRuntimeBundlePaths(bundleDir), []);
});
