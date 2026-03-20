import type {
  PluginHostBootstrap,
  PluginHostBootstrapInput,
} from "./types";
import { normalizeRegistrationMode } from "./registration-mode";

export { normalizeRegistrationMode } from "./registration-mode";

function normalizeRequiredPath(value: string, field: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw new Error(`${field} is required`);
  }
  return normalized;
}

export function createPluginHostBootstrap(
  input: PluginHostBootstrapInput,
): PluginHostBootstrap {
  return {
    pluginRootDir: normalizeRequiredPath(input.pluginRootDir, "pluginRootDir"),
    installRootDir: normalizeRequiredPath(input.installRootDir, "installRootDir"),
    registrationMode: normalizeRegistrationMode(input.registrationMode),
    runtimeBridge: input.runtimeBridge,
  };
}
