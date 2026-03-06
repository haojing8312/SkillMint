import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const projectRoot = process.cwd();
const tauriConfigPath = path.join(
  projectRoot,
  "apps",
  "runtime",
  "src-tauri",
  "tauri.conf.json",
);
const cargoTomlPath = path.join(
  projectRoot,
  "apps",
  "runtime",
  "src-tauri",
  "Cargo.toml",
);

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function readText(filePath) {
  return readFileSync(filePath, "utf8");
}

test("tauri updater is configured for desktop releases", () => {
  const config = readJson(tauriConfigPath);
  const cargoToml = readText(cargoTomlPath);
  const plugins = config?.plugins ?? {};
  const updater = plugins.updater;

  assert.ok(updater, "Expected plugins.updater to be configured in tauri.conf.json");
  assert.equal(
    typeof updater.pubkey,
    "string",
    "Expected plugins.updater.pubkey to be a non-empty string",
  );
  assert.ok(updater.pubkey.trim().length > 0, "Expected plugins.updater.pubkey to be non-empty");
  assert.ok(
    Array.isArray(updater.endpoints) && updater.endpoints.length > 0,
    "Expected plugins.updater.endpoints to be configured",
  );
  assert.match(
    cargoToml,
    /tauri-plugin-updater\s*=/,
    "Expected tauri-plugin-updater dependency in Cargo.toml",
  );
});
