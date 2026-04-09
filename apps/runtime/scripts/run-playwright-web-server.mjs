import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

function resolvePnpmRunner(env = process.env, platform = process.platform) {
  if (env.npm_execpath) {
    return {
      command: process.execPath,
      args: [env.npm_execpath],
    };
  }

  return {
    command: platform === "win32" ? "pnpm.cmd" : "pnpm",
    args: [],
  };
}

function main() {
  const runtimeRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const port = Number(process.env.PLAYWRIGHT_PORT || 4174);
  const runner = resolvePnpmRunner(process.env, process.platform);
  const child = spawn(
    runner.command,
    [
      ...runner.args,
      "exec",
      "vite",
      "--config",
      "e2e/vite.e2e.config.ts",
      "--host",
      "127.0.0.1",
      "--port",
      String(Number.isFinite(port) && port > 0 ? port : 4174),
      "--strictPort",
    ],
    {
      cwd: runtimeRoot,
      env: process.env,
      stdio: "inherit",
      windowsHide: false,
    },
  );

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code ?? 0);
  });
}

main();
