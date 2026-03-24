import test from "node:test";
import assert from "node:assert/strict";

import { buildDeployCommand, isRetryableWindowsDeployError } from "./prepare-sidecar-runtime-bundle.mjs";

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
