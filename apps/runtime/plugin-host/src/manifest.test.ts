import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  loadPluginManifestFromRoot,
  loadPluginPackageMetadataFromRoot,
  resolvePluginLoadEntries,
} from "./manifest";

const tempRoots: string[] = [];

async function createTempPluginRoot(): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "workclaw-plugin-manifest-"));
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

describe("plugin manifest loading", () => {
  it("reads openclaw.plugin.json and normalizes core fields", async () => {
    const root = await createTempPluginRoot();
    await writeJsonFile(root, "openclaw.plugin.json", {
      id: "openclaw-lark",
      channels: ["feishu"],
      skills: ["./skills"],
      configSchema: {
        type: "object",
        properties: {},
      },
    });

    const manifest = await loadPluginManifestFromRoot(root);

    expect(manifest.id).toBe("openclaw-lark");
    expect(manifest.channels).toEqual(["feishu"]);
    expect(manifest.skills).toEqual(["./skills"]);
    expect(manifest.configSchema).toEqual({
      type: "object",
      properties: {},
    });
  });

  it("reads package.json openclaw metadata and resolves setup entry", async () => {
    const root = await createTempPluginRoot();
    await writeJsonFile(root, "package.json", {
      name: "@larksuite/openclaw-lark",
      version: "2026.3.17",
      description: "OpenClaw Lark/Feishu channel plugin",
      openclaw: {
        extensions: ["./index.ts"],
        setupEntry: "./setup.ts",
        install: {
          npmSpec: "@larksuite/openclaw-lark",
          localPath: "extensions/feishu",
          defaultChoice: "npm",
        },
      },
    });

    const metadata = await loadPluginPackageMetadataFromRoot(root);

    expect(metadata.packageName).toBe("@larksuite/openclaw-lark");
    expect(metadata.version).toBe("2026.3.17");
    expect(metadata.description).toBe("OpenClaw Lark/Feishu channel plugin");
    expect(metadata.openclaw?.setupEntry).toBe("./setup.ts");
    expect(metadata.openclaw?.install?.npmSpec).toBe("@larksuite/openclaw-lark");
  });

  it("falls back to package extensions and then default index candidates", async () => {
    const root = await createTempPluginRoot();
    await writeJsonFile(root, "package.json", {
      name: "@larksuite/openclaw-lark",
      openclaw: {
        extensions: ["./index.ts"],
      },
    });

    const packageMetadata = await loadPluginPackageMetadataFromRoot(root);

    expect(resolvePluginLoadEntries(packageMetadata.openclaw)).toEqual(["./index.ts"]);
    expect(resolvePluginLoadEntries(undefined)).toContain("index.ts");
  });

  it("rejects manifests missing required fields", async () => {
    const root = await createTempPluginRoot();
    await writeJsonFile(root, "openclaw.plugin.json", {
      channels: ["feishu"],
      configSchema: {
        type: "object",
      },
    });

    await expect(loadPluginManifestFromRoot(root)).rejects.toThrow(/requires id/i);
  });
});
