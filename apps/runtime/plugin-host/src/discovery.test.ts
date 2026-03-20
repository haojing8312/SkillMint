import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { discoverInstalledPlugins } from "./discovery";

const tempRoots: string[] = [];

async function createTempDir(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempRoots.push(root);
  return root;
}

async function writeJsonFile(root: string, fileName: string, value: unknown): Promise<void> {
  await fs.writeFile(path.join(root, fileName), JSON.stringify(value, null, 2), "utf8");
}

afterEach(async () => {
  await Promise.all(
    tempRoots.splice(0, tempRoots.length).map((root) => fs.rm(root, { recursive: true, force: true })),
  );
});

describe("plugin discovery", () => {
  it("discovers plugin roots with manifest and package metadata", async () => {
    const installRoot = await createTempDir("workclaw-plugin-discovery-");
    const pluginRoot = path.join(installRoot, "openclaw-lark");
    await fs.mkdir(pluginRoot, { recursive: true });
    await writeJsonFile(pluginRoot, "openclaw.plugin.json", {
      id: "openclaw-lark",
      channels: ["feishu"],
      configSchema: { type: "object", properties: {} },
    });
    await writeJsonFile(pluginRoot, "package.json", {
      name: "@larksuite/openclaw-lark",
      version: "2026.3.17",
      openclaw: {
        extensions: ["./index.ts"],
        setupEntry: "./setup.ts",
        install: { npmSpec: "@larksuite/openclaw-lark" },
      },
    });

    const result = await discoverInstalledPlugins({
      installRoot,
    });

    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("openclaw-lark");
    expect(result[0].rootDir).toBe(pluginRoot);
    expect(result[0].entrypoints).toEqual(["./index.ts"]);
    expect(result[0].setupEntry).toBe("./setup.ts");
    expect(result[0].install?.npmSpec).toBe("@larksuite/openclaw-lark");
  });

  it("skips directories without a plugin manifest", async () => {
    const installRoot = await createTempDir("workclaw-plugin-discovery-empty-");
    await fs.mkdir(path.join(installRoot, "not-a-plugin"), { recursive: true });

    const result = await discoverInstalledPlugins({
      installRoot,
    });

    expect(result).toEqual([]);
  });
});
