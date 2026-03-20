import type { PluginRegistry } from "./registry";

export type ChannelPluginSummary = {
  id?: string;
  meta?: {
    id?: string;
    label?: string;
    selectionLabel?: string;
    docsPath?: string;
    docsLabel?: string;
    blurb?: string;
    aliases?: string[];
    order?: number;
  };
  capabilities?: Record<string, unknown>;
  reloadConfigPrefixes: string[];
  hasPairing: boolean;
  hasSetup: boolean;
  hasOnboarding: boolean;
  hasDirectory: boolean;
  hasOutbound: boolean;
  hasThreading: boolean;
  hasActions: boolean;
  hasStatus: boolean;
  targetHint?: string;
};

export type ToolSummary = {
  id?: string;
  name?: string;
  title?: string;
  description?: string;
};

export type RegistryInspectionSummary = {
  channels: ChannelPluginSummary[];
  tools: ToolSummary[];
  commandNames: string[];
  cliCommandNames: string[];
  gatewayMethods: string[];
  hookCounts: Record<string, number>;
};

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return value != null && typeof value === "object" ? (value as Record<string, unknown>) : undefined;
}

function readString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value : undefined;
}

function readStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string" && item.trim().length > 0);
}

export function summarizeChannelPlugin(channel: unknown): ChannelPluginSummary {
  const record = asRecord(channel) ?? {};
  const meta = asRecord(record.meta);
  const reload = asRecord(record.reload);
  const messaging = asRecord(record.messaging);
  const targetResolver = asRecord(messaging?.targetResolver);

  return {
    id: readString(record.id),
    meta: meta
      ? {
          id: readString(meta.id),
          label: readString(meta.label),
          selectionLabel: readString(meta.selectionLabel),
          docsPath: readString(meta.docsPath),
          docsLabel: readString(meta.docsLabel),
          blurb: readString(meta.blurb),
          aliases: readStringArray(meta.aliases),
          order: typeof meta.order === "number" ? meta.order : undefined,
        }
      : undefined,
    capabilities: asRecord(record.capabilities),
    reloadConfigPrefixes: readStringArray(reload?.configPrefixes),
    hasPairing: Boolean(record.pairing),
    hasSetup: Boolean(record.setup),
    hasOnboarding: Boolean(record.onboarding),
    hasDirectory: Boolean(record.directory),
    hasOutbound: Boolean(record.outbound),
    hasThreading: Boolean(record.threading),
    hasActions: Boolean(record.actions),
    hasStatus: Boolean(record.status),
    targetHint: readString(targetResolver?.hint),
  };
}

export function summarizeTool(tool: unknown): ToolSummary {
  const record = asRecord(tool) ?? {};
  return {
    id: readString(record.id),
    name: readString(record.name),
    title: readString(record.title),
    description: readString(record.description),
  };
}

export function summarizeRegistry(registry: PluginRegistry): RegistryInspectionSummary {
  const commandNames = registry.commands
    .map((command) => {
      const record = asRecord(command);
      return readString(record?.name) ?? readString(record?.id) ?? readString(record?.command);
    })
    .filter((value): value is string => Boolean(value));

  const cliCommandNames = registry.cliEntries
    .map((entry) => {
      const record = asRecord(entry);
      const registration = asRecord(record?.registration);
      return readStringArray(registration?.commands);
    })
    .flat();

  return {
    channels: registry.channels.map(summarizeChannelPlugin),
    tools: registry.tools.map(summarizeTool),
    commandNames,
    cliCommandNames,
    gatewayMethods: Object.keys(registry.gatewayMethods),
    hookCounts: Object.fromEntries(
      Object.entries(registry.hooks).map(([eventName, handlers]) => [eventName, handlers.length]),
    ),
  };
}
