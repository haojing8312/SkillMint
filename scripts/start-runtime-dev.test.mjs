import test from "node:test";
import assert from "node:assert/strict";
import process from "node:process";

import {
  buildRuntimeDevEnv,
  getRuntimeDevCommand,
} from "./start-runtime-dev.mjs";

test("runtime dev command targets the runtime package directly", () => {
  assert.deepEqual(getRuntimeDevCommand("win32", { npm_execpath: "C:\\pnpm\\pnpm.cjs" }), {
    command: process.execPath,
    args: ["C:\\pnpm\\pnpm.cjs", "--dir", "apps/runtime", "tauri", "dev"],
  });
});

test("runtime dev command falls back to platform pnpm binary without npm_execpath", () => {
  assert.deepEqual(getRuntimeDevCommand("win32", {}), {
    command: "pnpm.cmd",
    args: ["--dir", "apps/runtime", "tauri", "dev"],
  });
});

test("buildRuntimeDevEnv prepends cargo and rustup bins when cargo is missing from PATH", () => {
  const env = buildRuntimeDevEnv({
    env: {
      PATH: "C:\\Windows\\System32",
      CARGO_HOME: "D:\\worksoftdata\\.cargo",
      RUSTUP_HOME: "D:\\worksoftdata\\.rustup",
    },
    platform: "win32",
    pathDelimiter: ";",
    run: (command, args) => {
      if (command === "cargo.exe") {
        return { ok: false, stdout: "", stderr: "" };
      }
      if (command === "D:\\worksoftdata\\.cargo\\bin\\rustup.exe") {
        assert.deepEqual(args, ["which", "cargo"]);
        return {
          ok: true,
          stdout: "D:\\worksoftdata\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\bin\\cargo.exe\n",
          stderr: "",
        };
      }
      throw new Error(`Unexpected command: ${command}`);
    },
    exists: (candidate) =>
      candidate === "D:\\worksoftdata\\.cargo\\bin\\rustup.exe" ||
      candidate === "D:\\worksoftdata\\.cargo\\bin\\cargo.exe",
  });

  assert.match(
    env.PATH,
    /^D:\\worksoftdata\\\.cargo\\bin;D:\\worksoftdata\\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\bin;C:\\Windows\\System32$/,
  );
});

test("buildRuntimeDevEnv leaves PATH unchanged when cargo already resolves", () => {
  const env = buildRuntimeDevEnv({
    env: {
      PATH: "/usr/local/bin:/usr/bin",
    },
    platform: "linux",
    pathDelimiter: ":",
    run: (command) => {
      assert.equal(command, "cargo");
      return { ok: true, stdout: "/usr/bin/cargo\n", stderr: "" };
    },
    exists: () => false,
  });

  assert.equal(env.PATH, "/usr/local/bin:/usr/bin");
});
