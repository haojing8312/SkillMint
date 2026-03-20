export type PluginHookName = "before_tool_call" | "after_tool_call";

export type PluginHookHandler = (payload: unknown) => unknown | Promise<unknown>;

export type PluginRegistry = {
  channels: unknown[];
  tools: unknown[];
  cliEntries: unknown[];
  commands: unknown[];
  gatewayMethods: Record<string, unknown>;
  hooks: Record<PluginHookName, PluginHookHandler[]>;
};

export function createPluginRegistry(): PluginRegistry {
  return {
    channels: [],
    tools: [],
    cliEntries: [],
    commands: [],
    gatewayMethods: {},
    hooks: {
      before_tool_call: [],
      after_tool_call: [],
    },
  };
}
