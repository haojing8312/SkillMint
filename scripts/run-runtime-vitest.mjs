import { spawn, spawnSync } from "node:child_process";
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

function pnpmBinary(platform) {
  return platform === "win32" ? "pnpm.cmd" : "pnpm";
}

function resolvePnpmRunner({ env, platform }) {
  if (env.npm_execpath) {
    return {
      command: process.execPath,
      args: [env.npm_execpath],
    };
  }

  return {
    command: pnpmBinary(platform),
    args: [],
  };
}

export function resolveWorkspaceRoot(currentWorkingDirectory = process.cwd()) {
  const normalized = path.resolve(currentWorkingDirectory);
  if (path.basename(normalized) === "runtime" && path.basename(path.dirname(normalized)) === "apps") {
    return path.dirname(path.dirname(normalized));
  }
  return normalized;
}

export function buildVitestCommand(platform = process.platform, cliArgs = [], env = process.env) {
  const runner = resolvePnpmRunner({ env, platform });
  return {
    command: runner.command,
    args: [...runner.args, "--dir", "apps/runtime", "exec", "vitest", ...cliArgs],
  };
}

export function collectProcessTree(processes, rootPids = []) {
  const processMap = new Map(processes.map((item) => [item.pid, item]));
  const victimIds = new Set();
  const queue = rootPids.filter((pid) => Number.isInteger(pid) && pid > 0);

  while (queue.length > 0) {
    const pid = queue.shift();
    if (!pid || victimIds.has(pid)) continue;
    const current = processMap.get(pid);
    if (!current) continue;
    victimIds.add(pid);
    for (const processInfo of processes) {
      if (processInfo.parentPid === pid && !victimIds.has(processInfo.pid)) {
        queue.push(processInfo.pid);
      }
    }
  }

  return processes.filter((item) => victimIds.has(item.pid)).sort((left, right) => left.pid - right.pid);
}

export function getPidDirectoryPath(workspaceRoot) {
  return path.join(workspaceRoot, "temp", "runtime-vitest-pids");
}

export function getSessionPidFilePath(workspaceRoot, wrapperPid) {
  return path.join(getPidDirectoryPath(workspaceRoot), `${wrapperPid}.json`);
}

export function parseSessionPidFile(raw) {
  try {
    const parsed = JSON.parse(String(raw || ""));
    const wrapperPid = Number(parsed?.wrapperPid);
    const rootPids = Array.isArray(parsed?.rootPids) ? parsed.rootPids : [];
    if (!Number.isInteger(wrapperPid) || wrapperPid <= 0) {
      return null;
    }
    return {
      wrapperPid,
      rootPids: rootPids
      .map((item) => Number(item))
      .filter((item) => Number.isInteger(item) && item > 0),
    };
  } catch {
    return null;
  }
}

function readSessionPidFile(pidFilePath) {
  try {
    return parseSessionPidFile(readFileSync(pidFilePath, "utf8"));
  } catch {
    return null;
  }
}

function writeSessionPidFile(pidFilePath, session) {
  mkdirSync(path.dirname(pidFilePath), { recursive: true });
  writeFileSync(pidFilePath, JSON.stringify(session, null, 2));
}

function clearSessionPidFile(pidFilePath) {
  rmSync(pidFilePath, { force: true });
}

function readRecordedSessions(pidDirectoryPath) {
  try {
    return readdirSync(pidDirectoryPath)
      .filter((entry) => entry.endsWith(".json"))
      .map((entry) => ({
        filePath: path.join(pidDirectoryPath, entry),
        session: readSessionPidFile(path.join(pidDirectoryPath, entry)),
      }))
      .filter((item) => item.session !== null);
  } catch {
    return [];
  }
}

export function selectRecordedCleanupVictims(processes, recordedSessions, { excludeWrapperPids = [] } = {}) {
  const activeProcessIds = new Set(processes.map((item) => item.pid));
  const staleRootPids = recordedSessions
    .filter((item) => !excludeWrapperPids.includes(item.wrapperPid))
    .filter((item) => !activeProcessIds.has(item.wrapperPid))
    .flatMap((item) => item.rootPids);
  return collectProcessTree(processes, staleRootPids);
}

function listWindowsProcesses() {
  const script = [
    "$ErrorActionPreference='Stop'",
    "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,CommandLine | ConvertTo-Json -Depth 2 -Compress",
  ].join("; ");
  const result = spawnSync("powershell", ["-NoProfile", "-Command", script], {
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.status !== 0) {
    throw new Error(result.stderr.trim() || result.stdout.trim() || "unable to list Windows processes");
  }

  const parsed = JSON.parse(result.stdout.trim() || "[]");
  const items = Array.isArray(parsed) ? parsed : [parsed];
  return items.map((item) => ({
    pid: Number(item.ProcessId),
    parentPid: Number(item.ParentProcessId || 0),
    commandLine: String(item.CommandLine || ""),
  }));
}

function stopWindowsProcessIds(processIds) {
  if (processIds.length === 0) {
    return;
  }
  const victimIds = [...processIds].sort((left, right) => right - left);
  spawnSync("powershell", ["-NoProfile", "-Command", `Stop-Process -Id ${victimIds.join(",")} -Force`], {
    encoding: "utf8",
    windowsHide: true,
  });
}

function cleanupRecordedWindowsVitestProcesses({ pidDirectoryPath, excludeWrapperPids = [] }) {
  const processes = listWindowsProcesses();
  const recordedEntries = readRecordedSessions(pidDirectoryPath);
  const recordedSessions = recordedEntries.map((item) => item.session);
  const victims = selectRecordedCleanupVictims(processes, recordedSessions, { excludeWrapperPids })
    .filter((item) => item.pid > 0);

  for (const entry of recordedEntries) {
    if (!entry.session) continue;
    const wrapperStillActive = processes.some((item) => item.pid === entry.session.wrapperPid);
    if (!wrapperStillActive && !excludeWrapperPids.includes(entry.session.wrapperPid)) {
      clearSessionPidFile(entry.filePath);
    }
  }

  if (victims.length > 0) {
    stopWindowsProcessIds(victims.map((item) => item.pid));
  }

  return victims;
}

function main() {
  const cliArgs = process.argv.slice(2);
  const workspaceRoot = resolveWorkspaceRoot(process.cwd());
  const pidDirectoryPath = getPidDirectoryPath(workspaceRoot);
  const pidFilePath = getSessionPidFilePath(workspaceRoot, process.pid);

  if (process.platform === "win32") {
    cleanupRecordedWindowsVitestProcesses({
      pidDirectoryPath,
      excludeWrapperPids: [process.pid],
    });
  }

  const runner = buildVitestCommand(process.platform, cliArgs, process.env);
  const child = spawn(runner.command, runner.args, {
    cwd: workspaceRoot,
    env: process.env,
    stdio: "inherit",
    windowsHide: false,
  });
  writeSessionPidFile(pidFilePath, {
    wrapperPid: process.pid,
    rootPids: [child.pid].filter((pid) => Number.isInteger(pid) && pid > 0),
  });

  let shuttingDown = false;

  const terminateChild = () => {
    if (shuttingDown) return;
    shuttingDown = true;
    if (process.platform === "win32") {
      const processes = listWindowsProcesses();
      const ownVictims = collectProcessTree(processes, [child.pid].filter((pid) => Number.isInteger(pid) && pid > 0));
      stopWindowsProcessIds(ownVictims.map((item) => item.pid));
      cleanupRecordedWindowsVitestProcesses({
        pidDirectoryPath,
        excludeWrapperPids: [],
      });
      clearSessionPidFile(pidFilePath);
      return;
    }
    child.kill("SIGTERM");
  };

  process.on("SIGINT", () => {
    terminateChild();
    process.exit(130);
  });
  process.on("SIGTERM", () => {
    terminateChild();
    process.exit(143);
  });

  child.on("exit", (code, signal) => {
    clearSessionPidFile(pidFilePath);
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code ?? 0);
  });
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  main();
}
