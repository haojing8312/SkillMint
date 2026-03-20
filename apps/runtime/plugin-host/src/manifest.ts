import fs from "node:fs/promises";
import path from "node:path";

const PLUGIN_MANIFEST_FILE = "openclaw.plugin.json";
const PACKAGE_MANIFEST_FILE = "package.json";

const DEFAULT_PLUGIN_ENTRY_CANDIDATES = [
  "index.ts",
  "index.js",
  "index.mjs",
  "index.cjs",
] as const;

export type PluginInstallMetadata = {
  npmSpec?: string;
  localPath?: string;
  defaultChoice?: "npm" | "local";
};

export type OpenClawPackageMetadata = {
  extensions?: string[];
  setupEntry?: string;
  install?: PluginInstallMetadata;
};

export type PluginPackageMetadata = {
  packageName?: string;
  version?: string;
  description?: string;
  openclaw?: OpenClawPackageMetadata;
};

export type PluginManifest = {
  id: string;
  channels: string[];
  skills: string[];
  configSchema: Record<string, unknown>;
};

function normalizeString(value: unknown): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }
  const normalized = value.trim();
  return normalized ? normalized : undefined;
}

function normalizeStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((entry) => (typeof entry === "string" ? entry.trim() : ""))
    .filter(Boolean);
}

function normalizeInstallMetadata(value: unknown): PluginInstallMetadata | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return undefined;
  }

  const npmSpec = normalizeString((value as Record<string, unknown>).npmSpec);
  const localPath = normalizeString((value as Record<string, unknown>).localPath);
  const defaultChoice = (value as Record<string, unknown>).defaultChoice;

  return {
    ...(npmSpec ? { npmSpec } : {}),
    ...(localPath ? { localPath } : {}),
    ...(defaultChoice === "npm" || defaultChoice === "local" ? { defaultChoice } : {}),
  };
}

async function readJsonFile(rootDir: string, fileName: string): Promise<unknown> {
  const raw = await fs.readFile(path.join(rootDir, fileName), "utf8");
  return JSON.parse(raw) as unknown;
}

export async function loadPluginManifestFromRoot(rootDir: string): Promise<PluginManifest> {
  const raw = await readJsonFile(rootDir, PLUGIN_MANIFEST_FILE);
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    throw new Error("plugin manifest must be an object");
  }

  const manifest = raw as Record<string, unknown>;
  const id = normalizeString(manifest.id);
  if (!id) {
    throw new Error("plugin manifest requires id");
  }

  const configSchema = manifest.configSchema;
  if (!configSchema || typeof configSchema !== "object" || Array.isArray(configSchema)) {
    throw new Error("plugin manifest requires configSchema");
  }

  return {
    id,
    channels: normalizeStringArray(manifest.channels),
    skills: normalizeStringArray(manifest.skills),
    configSchema: configSchema as Record<string, unknown>,
  };
}

export async function loadPluginPackageMetadataFromRoot(
  rootDir: string,
): Promise<PluginPackageMetadata> {
  const raw = await readJsonFile(rootDir, PACKAGE_MANIFEST_FILE);
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    throw new Error("package manifest must be an object");
  }

  const manifest = raw as Record<string, unknown>;
  const openclawRaw = manifest.openclaw;
  let openclaw: OpenClawPackageMetadata | undefined;

  if (openclawRaw && typeof openclawRaw === "object" && !Array.isArray(openclawRaw)) {
    const openclawManifest = openclawRaw as Record<string, unknown>;
    const extensions = normalizeStringArray(openclawManifest.extensions);
    const setupEntry = normalizeString(openclawManifest.setupEntry);
    const install = normalizeInstallMetadata(openclawManifest.install);

    openclaw = {
      ...(extensions.length > 0 ? { extensions } : {}),
      ...(setupEntry ? { setupEntry } : {}),
      ...(install && Object.keys(install).length > 0 ? { install } : {}),
    };
  }

  return {
    packageName: normalizeString(manifest.name),
    version: normalizeString(manifest.version),
    description: normalizeString(manifest.description),
    ...(openclaw ? { openclaw } : {}),
  };
}

export function resolvePluginLoadEntries(
  metadata?: OpenClawPackageMetadata,
): string[] {
  const entries = metadata?.extensions?.map((entry) => entry.trim()).filter(Boolean) ?? [];
  if (entries.length > 0) {
    return entries;
  }
  return [...DEFAULT_PLUGIN_ENTRY_CANDIDATES];
}
