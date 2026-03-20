import path from "node:path";
import type { PluginRegistrationMode } from "./types";

export const REGISTRATION_MODES: PluginRegistrationMode[] = [
  "full",
  "setup-only",
  "setup-runtime",
];

export type ResolvePluginEntryPathInput = {
  rootDir: string;
  entrypoints: string[];
  registrationMode?: string;
  setupEntry?: string;
};

function normalizeRelativeEntryPath(entryPath?: string): string | undefined {
  const normalized = entryPath?.trim();
  if (!normalized) {
    return undefined;
  }
  return normalized;
}

export function normalizeRegistrationMode(value?: string): PluginRegistrationMode {
  const normalized = value?.trim();
  if (normalized && REGISTRATION_MODES.includes(normalized as PluginRegistrationMode)) {
    return normalized as PluginRegistrationMode;
  }
  return "full";
}

export function resolvePluginEntryPath(input: ResolvePluginEntryPathInput): string {
  const setupEntry = normalizeRelativeEntryPath(input.setupEntry);
  const primaryEntry = input.entrypoints.map((entry) => entry.trim()).find(Boolean);

  if (!primaryEntry) {
    throw new Error("plugin entrypoints are required");
  }

  const registrationMode = normalizeRegistrationMode(input.registrationMode);
  const selectedEntry =
    registrationMode === "full" || !setupEntry ? primaryEntry : setupEntry;

  return path.resolve(input.rootDir, selectedEntry);
}
