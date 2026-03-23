import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const projectRoot = process.cwd();
const packageJsonPath = path.join(projectRoot, "package.json");
const scriptPath = path.join(projectRoot, "scripts", "report-frontend-large-files.mjs");

test("package exposes a frontend large file reporting script", () => {
  const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));

  assert.equal(
    typeof pkg.scripts?.["report:frontend-large-files"],
    "string",
    "Expected a root report:frontend-large-files script",
  );
  assert.match(
    pkg.scripts["report:frontend-large-files"],
    /report-frontend-large-files\.mjs/,
    "Expected report:frontend-large-files to delegate to the shared script",
  );
});

test("frontend large file report script uses the documented thresholds and runtime scope", () => {
  const script = readFileSync(scriptPath, "utf8");

  assert.match(script, /DEFAULT_WARN_LINES = 300/, "Expected warn threshold of 300 lines");
  assert.match(script, /DEFAULT_PLAN_LINES = 500/, "Expected split-plan threshold of 500 lines");
  assert.match(
    script,
    /apps", "runtime", "src"/,
    "Expected script to scope reporting to the runtime frontend tree",
  );
  assert.match(script, /Thresholds must satisfy warn < plan/, "Expected threshold validation");
});
