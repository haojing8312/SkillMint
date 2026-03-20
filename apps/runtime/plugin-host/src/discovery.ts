import fs from "node:fs/promises";
import path from "node:path";
import {
  loadPluginManifestFromRoot,
  loadPluginPackageMetadataFromRoot,
  resolvePluginLoadEntries,
  type PluginInstallMetadata,
  type PluginManifest,
  type PluginPackageMetadata,
} from "./manifest";

export type DiscoverInstalledPluginsInput = {
  installRoot: string;
};

export type DiscoveredPlugin = {
  id: string;
  rootDir: string;
  manifest: PluginManifest;
  packageMetadata?: PluginPackageMetadata;
  entrypoints: string[];
  setupEntry?: string;
  install?: PluginInstallMetadata;
};

async function directoryExists(targetPath: string): Promise<boolean> {
  try {
    const stat = await fs.stat(targetPath);
    return stat.isDirectory();
  } catch {
    return false;
  }
}

export async function discoverInstalledPlugins(
  input: DiscoverInstalledPluginsInput,
): Promise<DiscoveredPlugin[]> {
  if (!(await directoryExists(input.installRoot))) {
    return [];
  }

  const entries = await fs.readdir(input.installRoot, { withFileTypes: true });
  const discovered: DiscoveredPlugin[] = [];

  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }

    const rootDir = path.join(input.installRoot, entry.name);

    try {
      const manifest = await loadPluginManifestFromRoot(rootDir);
      let packageMetadata: PluginPackageMetadata | undefined;

      try {
        packageMetadata = await loadPluginPackageMetadataFromRoot(rootDir);
      } catch {
        packageMetadata = undefined;
      }

      discovered.push({
        id: manifest.id,
        rootDir,
        manifest,
        packageMetadata,
        entrypoints: resolvePluginLoadEntries(packageMetadata?.openclaw),
        setupEntry: packageMetadata?.openclaw?.setupEntry,
        install: packageMetadata?.openclaw?.install,
      });
    } catch {
      continue;
    }
  }

  return discovered;
}
