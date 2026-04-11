import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { runRepoHygieneReview } from "./review-repo-hygiene.mjs";

const projectRoot = process.cwd();
const scriptPath = path.join(projectRoot, "scripts", "review-repo-hygiene.mjs");

test("review-repo-hygiene writes summary and json outputs for a supported mode", async () => {
  const outputDir = await mkdtemp(path.join(os.tmpdir(), "repo-hygiene-"));
  try {
    const proc = spawn(process.execPath, [
      scriptPath,
      "--output-dir",
      outputDir,
      "--mode",
      "artifacts",
    ], {
      cwd: projectRoot,
      stdio: "pipe",
    });

    let stderr = "";
    proc.stderr.on("data", (chunk) => {
      stderr += String(chunk);
    });

    const exitCode = await new Promise((resolve, reject) => {
      proc.on("error", reject);
      proc.on("close", resolve);
    });

    assert.equal(exitCode, 0, stderr);

    const summary = await readFile(path.join(outputDir, "summary.md"), "utf8");
    const report = JSON.parse(await readFile(path.join(outputDir, "report.json"), "utf8"));

    assert.match(summary, /Repo Hygiene Report/);
    assert.match(summary, /Mode: artifacts/);
    assert.equal(Array.isArray(report.findings), true);
    assert.equal(report.mode, "artifacts");
    assert.equal(typeof report.generatedAt, "string");
  } finally {
    await rm(outputDir, { recursive: true, force: true });
  }
});

test("review-repo-hygiene routes collectors by mode", async () => {
  const outputDir = await mkdtemp(path.join(os.tmpdir(), "repo-hygiene-"));
  try {
    const calls = [];
    await runRepoHygieneReview({
      outputDir,
      mode: "deadcode",
      collectors: {
        deadcode: async () => {
          calls.push("deadcode");
          return [
            {
              category: "dead-code",
              confidence: "confirmed",
              action: "ignore-with-rationale",
            },
          ];
        },
        artifacts: async () => {
          calls.push("artifacts");
          return [
            {
              category: "temporary-artifacts",
              confidence: "probable",
              action: "ignore-with-rationale",
            },
          ];
        },
        drift: async () => {
          calls.push("drift");
          return [
            {
              category: "stale-doc-or-skill-reference",
              confidence: "probable",
              action: "ignore-with-rationale",
            },
          ];
        },
      },
    });

    assert.deepEqual(calls, ["deadcode"]);
    const report = JSON.parse(await readFile(path.join(outputDir, "report.json"), "utf8"));
    assert.deepEqual(report.countsByCategory, { "dead-code": 1 });
  } finally {
    await rm(outputDir, { recursive: true, force: true });
  }
});
