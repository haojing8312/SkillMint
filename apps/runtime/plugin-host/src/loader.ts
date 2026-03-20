import fs from "node:fs/promises";
import crypto from "node:crypto";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { resolvePluginEntryPath } from "./registration-mode";
import type { PluginRegistrationMode } from "./types";

export type LoadPluginModuleInput = {
  rootDir: string;
  entrypoints: string[];
  registrationMode: PluginRegistrationMode;
  setupEntry?: string;
};

export type LoadedPluginModule = {
  entryPath: string;
  importPath: string;
  module: Record<string, unknown>;
};

type ImportRewriteMode = "node" | "vite";

function normalizeViteImportPath(absolutePath: string): string {
  const resolvedAbsolutePath = path.resolve(absolutePath);
  const normalizedAbsolutePath = resolvedAbsolutePath.replace(/\\/g, "/");

  if (!path.isAbsolute(resolvedAbsolutePath)) {
    return normalizedAbsolutePath.startsWith("/") ? normalizedAbsolutePath : `/${normalizedAbsolutePath}`;
  }

  const relativePath = path.relative(process.cwd(), resolvedAbsolutePath).replace(/\\/g, "/");
  const withinWorkspace =
    relativePath !== "" &&
    !relativePath.startsWith("../") &&
    relativePath !== ".." &&
    !path.isAbsolute(relativePath);

  if (withinWorkspace) {
    return relativePath.startsWith("/") ? relativePath : `/${relativePath}`;
  }

  return `/@fs/${normalizedAbsolutePath}`;
}

function resolvePluginSdkShimPath(
  shimPackageUrl: string,
  subpath: "" | "/compat" | "/feishu",
  mode: ImportRewriteMode,
): string {
  const packageDir = path.dirname(fileURLToPath(shimPackageUrl));
  const fileName =
    subpath === "/compat"
      ? "compat.js"
      : subpath === "/feishu"
        ? "feishu.js"
        : "index.js";
  const absolutePath = path.join(packageDir, "plugin-sdk", fileName);
  return mode === "vite"
    ? normalizeViteImportPath(absolutePath)
    : pathToFileURL(absolutePath).href;
}

function resolveRelativeImportPath(
  modulePath: string,
  specifier: string,
  mode: ImportRewriteMode,
): string {
  const absolutePath = path.resolve(path.dirname(modulePath), specifier);
  return mode === "vite"
    ? normalizeViteImportPath(absolutePath)
    : pathToFileURL(absolutePath).href;
}

async function materializeImportableEntry(params: {
  entryPath: string;
  sourceText: string;
}): Promise<string> {
  const cacheDir = path.join(process.cwd(), ".workclaw-plugin-host-cache");
  await fs.mkdir(cacheDir, { recursive: true });
  const digest = crypto
    .createHash("sha1")
    .update(params.entryPath)
    .update("\0")
    .update(params.sourceText)
    .digest("hex")
    .slice(0, 12);
  const baseName = path.basename(params.entryPath).replace(/\.[^.]+$/, "");
  const materializedPath = path.join(cacheDir, `${baseName}.${digest}.mjs`);
  await fs.writeFile(
    materializedPath,
    `${params.sourceText}\n//# sourceURL=${pathToFileURL(params.entryPath).href}\n`,
    "utf8",
  );
  return materializedPath;
}

async function ensurePluginSdkShimInstalled(rootDir: string, shimPackageUrl: string): Promise<void> {
  const shimPackagePath = fileURLToPath(shimPackageUrl);
  const shimPackageDir = path.dirname(shimPackagePath);
  const shimInstallDir = path.join(rootDir, "node_modules", "openclaw");
  await fs.mkdir(path.dirname(shimInstallDir), { recursive: true });
  await fs.rm(shimInstallDir, { recursive: true, force: true });
  await fs.cp(shimPackageDir, shimInstallDir, { recursive: true });
}

export function rewritePluginModuleImports(
  sourceText: string,
  shimPackageUrl?: string,
  modulePath?: string,
  mode: ImportRewriteMode = "node",
): string {
  let rewritten = sourceText;

  if (shimPackageUrl) {
    rewritten = rewritten
      .replaceAll(
        /(['"])openclaw\/plugin-sdk\/compat\1/g,
        (_match, quote) => `${quote}${resolvePluginSdkShimPath(shimPackageUrl, "/compat", mode)}${quote}`,
      )
      .replaceAll(
        /(['"])openclaw\/plugin-sdk\/feishu\1/g,
        (_match, quote) => `${quote}${resolvePluginSdkShimPath(shimPackageUrl, "/feishu", mode)}${quote}`,
      )
      .replaceAll(
        /(['"])openclaw\/plugin-sdk\1/g,
        (_match, quote) => `${quote}${resolvePluginSdkShimPath(shimPackageUrl, "", mode)}${quote}`,
      );
  }

  if (modulePath) {
    rewritten = rewritten
      .replaceAll(
        /from\s+(['"])(\.\.?\/[^'"]+)\1/g,
        (_match, quote, specifier) =>
          `from ${quote}${resolveRelativeImportPath(modulePath, specifier, mode)}${quote}`,
      )
      .replaceAll(
        /import\s+(['"])(\.\.?\/[^'"]+)\1/g,
        (_match, quote, specifier) =>
          `import ${quote}${resolveRelativeImportPath(modulePath, specifier, mode)}${quote}`,
      );
  }

  return rewritten;
}

export async function loadPluginModule(
  input: LoadPluginModuleInput & { shimPackageUrl?: string },
): Promise<LoadedPluginModule> {
  const entryPath = resolvePluginEntryPath(input);
  const rewriteMode: ImportRewriteMode = process.env.VITEST ? "vite" : "node";
  if (input.shimPackageUrl) {
    await ensurePluginSdkShimInstalled(input.rootDir, input.shimPackageUrl);
  }
  const originalSourceText = await fs.readFile(entryPath, "utf8");
  const sourceText = rewritePluginModuleImports(
    originalSourceText,
    input.shimPackageUrl,
    entryPath,
    rewriteMode,
  );
  const importPath = await materializeImportableEntry({
    entryPath,
    sourceText,
  });
  const importSpecifier =
    rewriteMode === "vite" ? normalizeViteImportPath(importPath) : pathToFileURL(importPath).href;
  const loadedModule = (await import(
    /* @vite-ignore */ importSpecifier
  )) as Record<string, unknown>;

  return {
    entryPath,
    importPath,
    module: loadedModule,
  };
}
