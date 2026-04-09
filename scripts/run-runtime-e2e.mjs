import { spawnSync } from "node:child_process";

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
  const runner = resolvePnpmRunner(process.env, process.platform);
  const forwardedArgs = process.argv.slice(2);
  const result = spawnSync(
    runner.command,
    [...runner.args, "--dir", "apps/runtime", "test:e2e", ...forwardedArgs],
    {
      cwd: process.cwd(),
      env: process.env,
      stdio: "inherit",
      windowsHide: false,
    },
  );

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status ?? 1);
}

main();
