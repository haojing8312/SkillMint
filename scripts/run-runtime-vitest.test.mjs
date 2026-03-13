import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";

import {
  buildVitestCommand,
  collectProcessTree,
  getPidDirectoryPath,
  getSessionPidFilePath,
  parseSessionPidFile,
  selectRecordedCleanupVictims,
  resolveWorkspaceRoot,
} from "./run-runtime-vitest.mjs";

test("collectProcessTree only returns descendants of recorded runtime pids", () => {
  const victims = collectProcessTree([
    {
      pid: 101,
      parentPid: 1,
      commandLine:
        '"node" "E:\\worksoftdata\\node\\node_gobal\\node_modules\\pnpm\\bin\\pnpm.cjs" exec vitest run src\\__tests__\\diagnostics.test.ts',
    },
    {
      pid: 102,
      parentPid: 101,
      commandLine:
        '"C:\\Windows\\System32\\cmd.exe" /c "cd /d e:\\code\\yzpd\\workclaw\\apps\\runtime && pnpm exec vitest run src\\__tests__\\diagnostics.test.ts"',
    },
    {
      pid: 103,
      parentPid: 101,
      commandLine:
        'node "E:\\code\\yzpd\\workclaw\\apps\\runtime\\node_modules\\.bin\\..\\vitest\\vitest.mjs" run src\\__tests__\\diagnostics.test.ts',
    },
    {
      pid: 201,
      parentPid: 2,
      commandLine:
        'node "C:\\other\\project\\node_modules\\vitest\\vitest.mjs" run smoke.test.ts',
    },
    {
      pid: 301,
      parentPid: 3,
      commandLine:
        'node "E:\\code\\yzpd\\workclaw\\apps\\runtime\\node_modules\\.bin\\vite.js" dev',
    },
  ], [101]);

  assert.deepEqual(
    victims.map((item) => item.pid),
    [101, 102, 103],
  );
});

test("collectProcessTree ignores unrelated concurrent vitest runs", () => {
  const victims = collectProcessTree([
    { pid: 11, parentPid: 1, commandLine: 'node "pnpm.cjs" exec vitest run a.test.ts' },
    { pid: 12, parentPid: 11, commandLine: 'node "vitest.mjs" run a.test.ts' },
    { pid: 21, parentPid: 2, commandLine: 'node "pnpm.cjs" exec vitest run b.test.ts' },
    { pid: 22, parentPid: 21, commandLine: 'node "vitest.mjs" run b.test.ts' },
  ], [21]);

  assert.deepEqual(
    victims.map((item) => item.pid),
    [21, 22],
  );
});

test("buildVitestCommand delegates to pnpm in the runtime package", () => {
  assert.deepEqual(buildVitestCommand("win32", ["run", "--passWithNoTests"], {
    npm_execpath: "C:\\pnpm\\pnpm.cjs",
  }), {
    command: process.execPath,
    args: ["C:\\pnpm\\pnpm.cjs", "--dir", "apps/runtime", "exec", "vitest", "run", "--passWithNoTests"],
  });
});

test("resolveWorkspaceRoot normalizes calls from the runtime package directory", () => {
  assert.equal(
    resolveWorkspaceRoot("E:\\code\\yzpd\\workclaw\\apps\\runtime"),
    "E:\\code\\yzpd\\workclaw",
  );
  assert.equal(
    resolveWorkspaceRoot("E:\\code\\yzpd\\workclaw"),
    "E:\\code\\yzpd\\workclaw",
  );
});

test("getPidDirectoryPath stores runtime vitest state under the workspace temp directory", () => {
  assert.equal(
    getPidDirectoryPath("E:\\code\\yzpd\\workclaw"),
    path.join("E:\\code\\yzpd\\workclaw", "temp", "runtime-vitest-pids"),
  );
});

test("getSessionPidFilePath uses the wrapper pid in the filename", () => {
  assert.equal(
    getSessionPidFilePath("E:\\code\\yzpd\\workclaw", 456),
    path.join("E:\\code\\yzpd\\workclaw", "temp", "runtime-vitest-pids", "456.json"),
  );
});

test("parseSessionPidFile tolerates malformed content", () => {
  assert.deepEqual(parseSessionPidFile("not-json"), null);
  assert.deepEqual(
    parseSessionPidFile(JSON.stringify({ wrapperPid: "45", rootPids: ["12", 13, -1, 0] })),
    { wrapperPid: 45, rootPids: [12, 13] },
  );
});

test("selectRecordedCleanupVictims only cleans stale sessions and preserves active concurrent wrappers", () => {
  const processes = [
    { pid: 777, parentPid: 1, commandLine: 'node "wrapper-a"' },
    { pid: 778, parentPid: 777, commandLine: 'node "vitest-a"' },
    { pid: 888, parentPid: 1, commandLine: 'node "wrapper-b"' },
    { pid: 889, parentPid: 888, commandLine: 'node "vitest-b"' },
  ];
  const sessions = [
    { wrapperPid: 777, rootPids: [778] },
    { wrapperPid: 999, rootPids: [889] },
  ];

  const victims = selectRecordedCleanupVictims(processes, sessions, { excludeWrapperPids: [888] });

  assert.deepEqual(
    victims.map((item) => item.pid),
    [889],
  );
});
