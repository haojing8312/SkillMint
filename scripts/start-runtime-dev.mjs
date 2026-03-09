import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

function runCommand(command, args, env) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    env,
    windowsHide: true,
  });

  return {
    ok: result.status === 0,
    stdout: result.stdout || "",
    stderr: result.stderr || "",
  };
}

function uniquePaths(paths, delimiter) {
  const seen = new Set();
  const ordered = [];

  for (const candidate of paths) {
    if (!candidate) continue;
    const normalized = candidate.trim();
    if (!normalized || seen.has(normalized)) continue;
    seen.add(normalized);
    ordered.push(normalized);
  }

  return ordered.join(delimiter);
}

function binaryName(base, platform) {
  return platform === "win32" ? `${base}.exe` : base;
}

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

function resolveRustupBinary({ env, platform, exists }) {
  const candidates = [];
  if (env.CARGO_HOME) {
    candidates.push(path.join(env.CARGO_HOME, "bin", binaryName("rustup", platform)));
  }
  if (env.RUSTUP_HOME) {
    candidates.push(path.join(env.RUSTUP_HOME, "bin", binaryName("rustup", platform)));
  }
  candidates.push(binaryName("rustup", platform));

  return candidates.find((candidate) => candidate === binaryName("rustup", platform) || exists(candidate)) || "";
}

function resolveCargoHomeBin({ env, platform, exists }) {
  if (!env.CARGO_HOME) {
    return "";
  }
  const cargoHomeBin = path.join(env.CARGO_HOME, "bin");
  const cargoBinary = path.join(cargoHomeBin, binaryName("cargo", platform));
  return exists(cargoBinary) ? cargoHomeBin : "";
}

function resolveCargoToolchainBin({ env, platform, run, exists }) {
  const rustupBinary = resolveRustupBinary({ env, platform, exists });
  if (!rustupBinary) {
    return "";
  }

  const result = run(rustupBinary, ["which", "cargo"], env);
  if (!result.ok) {
    return "";
  }

  const cargoBinary = result.stdout.trim();
  return cargoBinary ? path.dirname(cargoBinary) : "";
}

export function buildRuntimeDevEnv({
  env = process.env,
  platform = process.platform,
  pathDelimiter = path.delimiter,
  run = runCommand,
  exists = () => false,
} = {}) {
  const nextEnv = { ...env };
  const cargoCheck = run(binaryName("cargo", platform), ["--version"], nextEnv);
  if (cargoCheck.ok) {
    return nextEnv;
  }

  const prependedPaths = [];
  const cargoHomeBin = resolveCargoHomeBin({ env: nextEnv, platform, exists });
  if (cargoHomeBin) {
    prependedPaths.push(cargoHomeBin);
  }

  const cargoToolchainBin = resolveCargoToolchainBin({
    env: nextEnv,
    platform,
    run,
    exists,
  });
  if (cargoToolchainBin) {
    prependedPaths.push(cargoToolchainBin);
  }

  nextEnv.PATH = uniquePaths([...prependedPaths, nextEnv.PATH || ""], pathDelimiter);
  return nextEnv;
}

export function getRuntimeDevCommand(platform = process.platform, env = process.env) {
  const runner = resolvePnpmRunner({ env, platform });
  return {
    command: runner.command,
    args: [...runner.args, "--dir", "apps/runtime", "tauri", "dev"],
  };
}

function ensureBinary(name, args, env) {
  const result = runCommand(name, args, env);
  if (!result.ok) {
    const details = result.stderr.trim() || result.stdout.trim();
    throw new Error(`Unable to run ${name} ${args.join(" ")}${details ? `: ${details}` : ""}`);
  }
}

function main() {
  const env = buildRuntimeDevEnv({
    env: process.env,
    exists: existsSync,
  });

  const runner = resolvePnpmRunner({ env, platform: process.platform });
  ensureBinary(runner.command, [...runner.args, "--version"], env);
  ensureBinary(binaryName("cargo", process.platform), ["--version"], env);

  const { command, args } = getRuntimeDevCommand(process.platform, env);
  const child = spawn(command, args, {
    cwd: process.cwd(),
    env,
    stdio: "inherit",
    windowsHide: false,
  });

  child.on("exit", (code, signal) => {
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
