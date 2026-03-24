import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = fileURLToPath(import.meta.url);
const projectRoot = path.resolve(scriptDir, "..");
const bundleDir = path.join(
  projectRoot,
  "apps",
  "runtime",
  "src-tauri",
  "resources",
  "sidecar-runtime",
);
const bundledNodeName = process.platform === "win32" ? "node.exe" : "node";

function resolvePnpmRunner() {
  return {
    command: process.platform === "win32" ? "pnpm.cmd" : "pnpm",
    args: [],
  };
}

export function buildDeployCommand(runner, pnpmMajor, targetDir, baseEnv = process.env) {
  const deployArgs = [
    ...runner.args,
    "--filter",
    "workclaw-runtime-sidecar",
    "deploy",
    "--prod",
    "--config.bin-links=false",
  ];
  if (pnpmMajor >= 10) {
    deployArgs.push("--legacy");
  }
  deployArgs.push(targetDir);

  return {
    command: runner.command,
    args: deployArgs,
    env: {
      ...baseEnv,
      npm_config_bin_links: "false",
      pnpm_config_bin_links: "false",
      NPM_CONFIG_BIN_LINKS: "false",
      PNPM_CONFIG_BIN_LINKS: "false",
    },
  };
}

export function isRetryableWindowsDeployError(output, platform = process.platform) {
  if (platform !== "win32") {
    return false;
  }

  const text = String(output ?? "");
  return (
    text.includes("playwright.CMD") ||
    text.includes("playwright.ps1") ||
    text.includes("Failed to create bin at") ||
    text.includes("EPERM") ||
    text.includes("ENOENT: no such file or directory, chmod")
  );
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

function runOrThrow(command, args, env = process.env) {
  const result = spawnSync(command, args, {
    cwd: projectRoot,
    stdio: "pipe",
    encoding: "utf8",
    windowsHide: true,
    env,
    shell: process.platform === "win32",
  });

  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }

  if (result.status !== 0) {
    const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
    const error = new Error(`Command failed: ${command} ${args.join(" ")}`);
    error.cause = { output, status: result.status };
    throw error;
  }
}

function removeDirForWindowsBuild(targetDir) {
  if (process.platform === "win32") {
    spawnSync("cmd.exe", ["/c", "rmdir", "/s", "/q", targetDir], {
      cwd: projectRoot,
      windowsHide: true,
      env: process.env,
    });
  }
  rmSync(targetDir, { recursive: true, force: true });
}

function main() {
  const runner = resolvePnpmRunner();
  const pnpmMajor = readPnpmMajorVersion(runner);
  const deployCommand = buildDeployCommand(runner, pnpmMajor, bundleDir);
  let deployAttempt = 0;
  while (true) {
    deployAttempt += 1;
    removeDirForWindowsBuild(bundleDir);
    try {
      runOrThrow(deployCommand.command, deployCommand.args, deployCommand.env);
      break;
    } catch (error) {
      const output = error && typeof error === "object" && "cause" in error
        ? error.cause?.output
        : "";
      if (deployAttempt >= 2 || !isRetryableWindowsDeployError(output)) {
        throw error;
      }
      console.warn("Retrying sidecar runtime deploy after transient Windows bin creation failure...");
    }
  }

  const distEntry = path.join(bundleDir, "dist", "index.js");
  if (!existsSync(distEntry)) {
    throw new Error(
      `Bundled sidecar runtime is missing ${distEntry}. Run the sidecar build before staging resources.`,
    );
  }

  copyFileSync(process.execPath, path.join(bundleDir, bundledNodeName));
}

const isMainModule =
  typeof process.argv[1] === "string" &&
  path.resolve(process.argv[1]) === path.resolve(scriptPath);

if (isMainModule) {
  main();
}
