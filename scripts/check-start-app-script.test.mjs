import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const projectRoot = process.cwd();
const packageJsonPath = path.join(projectRoot, "package.json");
const startScriptPath = path.join(projectRoot, "scripts", "start-runtime-dev.mjs");
const cmdScriptPath = path.join(projectRoot, "tmp-start-app.cmd");
const vbsScriptPath = path.join(projectRoot, "tmp-start-app.vbs");

function readScript(scriptPath) {
  return readFileSync(scriptPath, "utf8");
}

test("root desktop app script uses the cross-platform runtime launcher", () => {
  const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));

  assert.equal(
    pkg.scripts?.app,
    "node scripts/start-runtime-dev.mjs",
    "Expected pnpm app to delegate to the shared runtime launcher",
  );
  assert.equal(
    pkg.scripts?.runtime,
    "node scripts/start-runtime-dev.mjs",
    "Expected pnpm runtime to delegate to the shared runtime launcher",
  );
});

test("runtime launcher prefers environment-driven cargo resolution over hardcoded paths", () => {
  const script = readScript(startScriptPath);

  assert.match(
    script,
    /\.CARGO_HOME/,
    "Expected launcher to read CARGO_HOME from the environment first",
  );
  assert.match(
    script,
    /\.RUSTUP_HOME/,
    "Expected launcher to read RUSTUP_HOME from the environment first",
  );
  assert.match(
    script,
    /\[\s*"which"\s*,\s*"cargo"\s*\]/i,
    "Expected launcher to fall back to rustup which cargo when PATH is incomplete",
  );
  assert.doesNotMatch(
    script,
    /tmp-start-app\.(cmd|vbs)/i,
    "Expected launcher not to depend on the Windows-only temporary startup helpers",
  );
});

test("local start cmd script derives Rust paths from environment", () => {
  const script = readScript(cmdScriptPath);

  assert.match(
    script,
    /if not defined CARGO_HOME set "CARGO_HOME=%USERPROFILE%\\\.cargo"/i,
    "Expected tmp-start-app.cmd to default CARGO_HOME from USERPROFILE",
  );
  assert.match(
    script,
    /if not defined RUSTUP_HOME set "RUSTUP_HOME=%USERPROFILE%\\\.rustup"/i,
    "Expected tmp-start-app.cmd to default RUSTUP_HOME from USERPROFILE",
  );
  assert.match(
    script,
    /set "PATH=%RUSTUP_HOME%\\toolchains\\stable-x86_64-pc-windows-msvc\\bin;%CARGO_HOME%\\bin;%PATH%"/i,
    "Expected tmp-start-app.cmd to prepend the active Rust toolchain and cargo bin to PATH",
  );
  assert.doesNotMatch(
    script,
    /C:\\Users\\36443\\\.(cargo|rustup)/i,
    "tmp-start-app.cmd should not hardcode a specific user Rust path",
  );
});

test("local start vbs script delegates to the cmd launcher", () => {
  const script = readScript(vbsScriptPath);

  assert.match(
    script,
    /tmp-start-app\.cmd/i,
    "Expected tmp-start-app.vbs to launch the cmd helper script",
  );
  assert.doesNotMatch(
    script,
    /C:\\Users\\36443\\\.(cargo|rustup)/i,
    "tmp-start-app.vbs should not hardcode a specific user Rust path",
  );
});
