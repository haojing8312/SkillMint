import type { PluginHookHandler, PluginHookName, PluginRegistry } from "./registry";
import { normalizeRegistrationMode } from "./registration-mode";

export type OpenClawPluginApiLike = {
  runtime: Record<string, unknown>;
  logger: Record<string, unknown>;
  config?: Record<string, unknown>;
  registrationMode: "full" | "setup-only" | "setup-runtime";
  registerChannel: (input: { plugin: unknown }) => void;
  registerTool: (tool: unknown) => void;
  registerCli: (cliEntry: unknown, options?: unknown) => void;
  registerGatewayMethod: (name: string, handler: unknown) => void;
  registerCommand: (command: unknown) => void;
  on: (eventName: PluginHookName, handler: PluginHookHandler) => void;
};

export function createPluginApi(
  registry: PluginRegistry,
  options?: {
    runtime?: Record<string, unknown>;
    logger?: Record<string, unknown>;
    config?: Record<string, unknown>;
    registrationMode?: string;
    createCliContext?: () => Record<string, unknown>;
  },
): OpenClawPluginApiLike {
  return {
    runtime: options?.runtime ?? {},
    logger: options?.logger ?? {},
    config: options?.config,
    registrationMode: normalizeRegistrationMode(options?.registrationMode),
    registerChannel(input) {
      registry.channels.push(input.plugin);
    },
    registerTool(tool) {
      registry.tools.push(tool);
    },
    registerCli(cliEntry, registration) {
      if (typeof cliEntry === "function") {
        cliEntry(options?.createCliContext?.() ?? {});
      }
      registry.cliEntries.push({
        entry: cliEntry,
        registration,
      });
    },
    registerGatewayMethod(name, handler) {
      registry.gatewayMethods[name] = handler;
    },
    registerCommand(command) {
      registry.commands.push(command);
    },
    on(eventName, handler) {
      registry.hooks[eventName].push(handler);
    },
  };
}
