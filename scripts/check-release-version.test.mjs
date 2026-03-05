import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";

const SCRIPT_PATH = path.resolve("scripts", "check-release-version.mjs");

async function makeFixture({ runtimeVersion, tauriVersion, cargoVersion }) {
  const root = await mkdtemp(path.join(os.tmpdir(), "workclaw-release-check-"));
  const runtimeDir = path.join(root, "apps", "runtime");
  const tauriDir = path.join(runtimeDir, "src-tauri");
  await mkdir(tauriDir, { recursive: true });

  await writeFile(
    path.join(runtimeDir, "package.json"),
    `${JSON.stringify({ name: "runtime", version: runtimeVersion }, null, 2)}\n`,
    "utf8",
  );

  await writeFile(
    path.join(tauriDir, "tauri.conf.json"),
    `${JSON.stringify({ version: tauriVersion }, null, 2)}\n`,
    "utf8",
  );

  await writeFile(
    path.join(tauriDir, "Cargo.toml"),
    `[package]\nname = "runtime"\nversion = "${cargoVersion}"\n`,
    "utf8",
  );

  return root;
}

function runCheck({ cwd, tag }) {
  return spawnSync(process.execPath, [SCRIPT_PATH], {
    cwd,
    env: { ...process.env, GITHUB_REF_NAME: tag },
    encoding: "utf8",
  });
}

test("passes when tag matches runtime/tauri/cargo versions", async () => {
  const fixture = await makeFixture({
    runtimeVersion: "0.2.0",
    tauriVersion: "0.2.0",
    cargoVersion: "0.2.0",
  });

  try {
    const result = runCheck({ cwd: fixture, tag: "v0.2.0" });
    assert.equal(result.status, 0, result.stderr || result.stdout);
  } finally {
    await rm(fixture, { recursive: true, force: true });
  }
});

test("fails when runtime package version mismatches tag", async () => {
  const fixture = await makeFixture({
    runtimeVersion: "0.1.9",
    tauriVersion: "0.2.0",
    cargoVersion: "0.2.0",
  });

  try {
    const result = runCheck({ cwd: fixture, tag: "v0.2.0" });
    assert.equal(result.status, 1, "Expected version check to fail");
    assert.match(result.stderr, /runtime\/package\.json/i);
  } finally {
    await rm(fixture, { recursive: true, force: true });
  }
});

test("fails when cargo package version mismatches tag", async () => {
  const fixture = await makeFixture({
    runtimeVersion: "0.2.0",
    tauriVersion: "0.2.0",
    cargoVersion: "0.1.8",
  });

  try {
    const result = runCheck({ cwd: fixture, tag: "v0.2.0" });
    assert.equal(result.status, 1, "Expected version check to fail");
    assert.match(result.stderr, /Cargo\.toml/i);
  } finally {
    await rm(fixture, { recursive: true, force: true });
  }
});
