import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(scriptDir, "..");
const bundleDir = path.join(
  projectRoot,
  "apps",
  "runtime",
  "src-tauri",
  "resources",
  "sidecar-runtime",
);
const distEntry = path.join(bundleDir, "dist", "index.js");
const bundledNodeName = process.platform === "win32" ? "node.exe" : "node";

function resolvePnpmRunner() {
  return {
    command: process.platform === "win32" ? "pnpm.cmd" : "pnpm",
    args: [],
  };
}

function readPnpmMajorVersion(runner) {
  const result = spawnSync(runner.command, [...runner.args, "--version"], {
    cwd: projectRoot,
    encoding: "utf8",
    windowsHide: true,
    env: process.env,
    shell: process.platform === "win32",
  });

  if (result.status !== 0) {
    throw new Error(`Unable to detect pnpm version via ${runner.command} --version`);
  }

  const versionText = String(result.stdout ?? "").trim();
  const major = Number.parseInt(versionText.split(".")[0] ?? "", 10);
  if (!Number.isFinite(major)) {
    throw new Error(`Unexpected pnpm version output: ${versionText}`);
  }
  return major;
}

function runOrThrow(command, args) {
  const result = spawnSync(command, args, {
    cwd: projectRoot,
    stdio: "inherit",
    windowsHide: true,
    env: process.env,
    shell: process.platform === "win32",
  });

  if (result.status !== 0) {
    throw new Error(`Command failed: ${command} ${args.join(" ")}`);
  }
}

function main() {
  rmSync(bundleDir, { recursive: true, force: true });

  const runner = resolvePnpmRunner();
  const pnpmMajor = readPnpmMajorVersion(runner);
  const deployArgs = [
    ...runner.args,
    "--filter",
    "workclaw-runtime-sidecar",
    "deploy",
    "--prod",
  ];
  if (pnpmMajor >= 10) {
    deployArgs.push("--legacy");
  }
  deployArgs.push("--config.bin-links=false", bundleDir);
  runOrThrow(runner.command, [
    ...deployArgs,
  ]);

  if (!existsSync(distEntry)) {
    throw new Error(
      `Bundled sidecar runtime is missing ${distEntry}. Run the sidecar build before staging resources.`,
    );
  }

  copyFileSync(process.execPath, path.join(bundleDir, bundledNodeName));
}

main();
