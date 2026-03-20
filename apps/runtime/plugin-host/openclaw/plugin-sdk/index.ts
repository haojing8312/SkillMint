export const DEFAULT_ACCOUNT_ID = "default";
export const DEFAULT_GROUP_HISTORY_LIMIT = 50;
export const PAIRING_APPROVED_MESSAGE =
  "Pairing approved. You can now message this bot directly.";
export const SILENT_REPLY_TOKEN = "NO_REPLY";

export type ReplyPayload = Record<string, unknown>;
export type HistoryEntry = Record<string, unknown>;
export type OpenClawConfig = Record<string, unknown>;
export type ClawdbotConfig = OpenClawConfig;
export type RuntimeEnv = Record<string, unknown>;
export type RuntimeLogger = {
  debug?: (...args: unknown[]) => void;
  info?: (...args: unknown[]) => void;
  warn?: (...args: unknown[]) => void;
  error?: (...args: unknown[]) => void;
};
export type PluginRuntime = Record<string, unknown>;
export type OpenClawPluginApi = Record<string, unknown>;
export type ChannelMeta = Record<string, unknown>;
export type ChannelPlugin<T = unknown> = Record<string, unknown> & { id?: string };
export type ChannelThreadingToolContext = Record<string, unknown>;
export type ChannelOutboundAdapter = Record<string, unknown>;
export type ChannelGroupContext = Record<string, unknown>;
export type GroupToolPolicyConfig = Record<string, unknown>;
export type DmPolicy = string;
export type WizardPrompter = Record<string, unknown>;

export function emptyPluginConfigSchema(): Record<string, unknown> {
  return {
    type: "object",
    additionalProperties: false,
    properties: {},
  };
}

export function normalizeAccountId(value?: string | null): string {
  const normalized = value?.trim();
  return normalized || DEFAULT_ACCOUNT_ID;
}

export function buildRandomTempFilePath(params: {
  fileName?: string;
  prefix?: string;
  extension?: string;
} = {}): string {
  const prefix = params.prefix?.trim() || "workclaw-plugin";
  const fileName = params.fileName?.trim();
  const extension = params.extension?.trim() || "";
  const suffix = `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  if (fileName) {
    return `${prefix}-${suffix}-${fileName}`;
  }
  return `${prefix}-${suffix}${extension}`;
}

export function addWildcardAllowFrom(
  allowFrom?: Array<string | number> | null,
): string[] {
  const normalized = (allowFrom ?? [])
    .map((entry) => String(entry).trim())
    .filter(Boolean);
  return normalized.includes("*") ? normalized : [...normalized, "*"];
}

export function formatDocsLink(path: string): string {
  const normalized = path.startsWith("/") ? path : `/${path}`;
  return `https://docs.openclaw.ai${normalized}`;
}

export function buildPendingHistoryContextFromMap(params: {
  historyMap?: Map<string, HistoryEntry[]>;
  historyKey?: string;
  limit?: number;
  currentMessage?: string;
  formatEntry?: (entry: HistoryEntry) => string;
  lineBreak?: string;
}): string {
  const historyMap = params.historyMap;
  const historyKey = params.historyKey?.trim();
  const limit = params.limit ?? 0;
  const currentMessage = String(params.currentMessage ?? "");
  if (!historyMap || !historyKey || limit <= 0) {
    return currentMessage;
  }
  const entries = (historyMap.get(historyKey) ?? []).slice(-limit);
  if (entries.length === 0) {
    return currentMessage;
  }
  const formatter =
    params.formatEntry ??
    ((entry: HistoryEntry) =>
      `${String((entry as Record<string, unknown>).sender ?? "user")}: ${String((entry as Record<string, unknown>).body ?? "")}`);
  const lineBreak = params.lineBreak ?? "\n";
  const historyText = entries.map(formatter).join(lineBreak);
  if (!historyText.trim()) {
    return currentMessage;
  }
  return [
    "[Chat messages since your last reply - for context]",
    historyText,
    "",
    "[Current message]",
    currentMessage,
  ].join(lineBreak);
}

export function resolveThreadSessionKeys(params: {
  baseSessionKey: string;
  threadId?: string | null;
  parentSessionKey?: string;
  useSuffix?: boolean;
  normalizeThreadId?: (threadId: string) => string;
}): { sessionKey: string; parentSessionKey?: string } {
  const baseSessionKey = params.baseSessionKey;
  const threadId = params.threadId?.trim();
  if (!threadId) {
    return {
      sessionKey: baseSessionKey,
      parentSessionKey: params.parentSessionKey,
    };
  }
  const normalizedThreadId = (params.normalizeThreadId ?? ((value: string) => value.toLowerCase()))(threadId);
  const useSuffix = params.useSuffix ?? true;
  return {
    sessionKey: useSuffix ? `${baseSessionKey}:thread:${normalizedThreadId}` : baseSessionKey,
    parentSessionKey: params.parentSessionKey,
  };
}

export function clearHistoryEntriesIfEnabled(params: {
  historyMap?: Map<string, HistoryEntry[]>;
  historyKey?: string;
  limit?: number;
}): void {
  if (!params.historyMap || !params.historyKey || (params.limit ?? 0) <= 0) {
    return;
  }
  params.historyMap.set(params.historyKey, []);
}

export function recordPendingHistoryEntryIfEnabled(params: {
  historyMap?: Map<string, HistoryEntry[]>;
  historyKey?: string;
  entry?: HistoryEntry | null;
  limit?: number;
}): HistoryEntry[] {
  const historyMap = params.historyMap;
  const historyKey = params.historyKey?.trim();
  const entry = params.entry;
  const limit = params.limit ?? 0;
  if (!historyMap || !historyKey || !entry || limit <= 0) {
    return [];
  }
  const history = historyMap.get(historyKey) ?? [];
  history.push(entry);
  while (history.length > limit) {
    history.shift();
  }
  historyMap.set(historyKey, history);
  return history;
}

export async function resolveSenderCommandAuthorization(
  params: {
    rawBody?: string;
    cfg?: Record<string, unknown>;
    isGroup?: boolean;
    dmPolicy?: string;
    configuredAllowFrom?: string[];
    configuredGroupAllowFrom?: string[];
    senderId?: string;
    isSenderAllowed?: (senderId: string, allowFrom: string[]) => boolean;
    readAllowFromStore?: () => Promise<string[]>;
    shouldComputeCommandAuthorized?: (rawBody: string, cfg: Record<string, unknown>) => boolean;
    resolveCommandAuthorizedFromAuthorizers?: (params: {
      useAccessGroups: boolean;
      authorizers: Array<{ configured: boolean; allowed: boolean }>;
    }) => boolean;
  },
): Promise<{
  shouldComputeAuth: boolean;
  effectiveAllowFrom: string[];
  effectiveGroupAllowFrom: string[];
  senderAllowedForCommands: boolean;
  commandAuthorized: boolean | undefined;
}> {
  const cfg = params.cfg ?? {};
  const rawBody = String(params.rawBody ?? "");
  const shouldComputeAuth = params.shouldComputeCommandAuthorized
    ? params.shouldComputeCommandAuthorized(rawBody, cfg)
    : rawBody.trim().startsWith("/");
  const configuredAllowFrom = params.configuredAllowFrom ?? [];
  const configuredGroupAllowFrom = params.configuredGroupAllowFrom ?? [];
  const storeAllowFrom = params.readAllowFromStore ? await params.readAllowFromStore().catch(() => []) : [];
  const effectiveAllowFrom = Array.from(new Set([...configuredAllowFrom, ...storeAllowFrom]));
  const effectiveGroupAllowFrom = Array.from(new Set(configuredGroupAllowFrom));
  const senderId = String(params.senderId ?? "");
  const isSenderAllowed =
    params.isSenderAllowed ??
    ((candidate: string, allowFrom: string[]) =>
      allowFrom.map((entry) => entry.trim().toLowerCase()).includes(candidate.trim().toLowerCase()));
  const senderAllowedForCommands = isSenderAllowed(
    senderId,
    params.isGroup ? effectiveGroupAllowFrom : effectiveAllowFrom,
  );
  const commandAuthorized = shouldComputeAuth
    ? params.resolveCommandAuthorizedFromAuthorizers
      ? params.resolveCommandAuthorizedFromAuthorizers({
          useAccessGroups: true,
          authorizers: [
            { configured: effectiveAllowFrom.length > 0, allowed: isSenderAllowed(senderId, effectiveAllowFrom) },
            {
              configured: effectiveGroupAllowFrom.length > 0,
              allowed: isSenderAllowed(senderId, effectiveGroupAllowFrom),
            },
          ],
        })
      : senderAllowedForCommands || (effectiveAllowFrom.length === 0 && effectiveGroupAllowFrom.length === 0)
    : undefined;

  return {
    shouldComputeAuth,
    effectiveAllowFrom,
    effectiveGroupAllowFrom,
    senderAllowedForCommands,
    commandAuthorized,
  };
}

export function isNormalizedSenderAllowed(params: {
  normalizedSenderId?: string | null;
  allowFrom?: Array<string | number> | null;
}): boolean {
  const normalizedSenderId = params.normalizedSenderId?.trim().toLowerCase();
  if (!normalizedSenderId) {
    return false;
  }
  const allowFrom = (params.allowFrom ?? []).map((entry) =>
    String(entry).trim().toLowerCase(),
  );
  return allowFrom.includes("*") || allowFrom.includes(normalizedSenderId);
}

export function extractToolSend(result: unknown): unknown {
  return result;
}

export function jsonResult(payload: unknown): { ok: true; payload: unknown } {
  return { ok: true, payload };
}

export function readStringParam(
  params: Record<string, unknown>,
  key: string,
): string | undefined {
  const value = params[key];
  return typeof value === "string" ? value : undefined;
}

export function readReactionParams(
  params: Record<string, unknown>,
): Record<string, unknown> {
  return params;
}

export function createReplyPrefixContext(params: {
  prefix?: string;
  text?: string;
  cfg?: Record<string, unknown>;
  agentId?: string;
  channel?: string;
  accountId?: string;
}): {
  prefixContext: Record<string, unknown>;
  responsePrefix?: string;
  enableSlackInteractiveReplies?: boolean;
  responsePrefixContextProvider: () => Record<string, unknown>;
  onModelSelected: (ctx: Record<string, unknown>) => void;
  prefixedText: string;
} {
  const prefix = params.prefix?.trim();
  const text = params.text ?? "";
  const prefixContext: Record<string, unknown> = {
    agentId: params.agentId ?? "main",
    channel: params.channel,
    accountId: params.accountId,
  };
  return {
    prefixedText: prefix ? `${prefix} ${text}`.trim() : String(text),
    prefixContext,
    responsePrefix: prefix,
    enableSlackInteractiveReplies: false,
    responsePrefixContextProvider: () => prefixContext,
    onModelSelected(ctx) {
      Object.assign(prefixContext, ctx);
    },
  };
}

export function createTypingCallbacks(params?: {
  start?: () => Promise<void>;
  stop?: () => Promise<void>;
  onStartError?: (err: unknown) => void;
  onStopError?: (err: unknown) => void;
}): {
  onReplyStart: () => Promise<void>;
  onIdle: () => void;
  onCleanup: () => void;
} {
  return {
    async onReplyStart() {
      if (!params?.start) {
        return;
      }
      try {
        await params.start();
      } catch (error) {
        params.onStartError?.(error);
      }
    },
    onIdle() {
      if (!params?.stop) {
        return;
      }
      void params.stop().catch((error) => params.onStopError?.(error));
    },
    onCleanup() {
      if (!params?.stop) {
        return;
      }
      void params.stop().catch((error) => params.onStopError?.(error));
    },
  };
}

export function logTypingFailure(params: {
  logger?: RuntimeLogger;
  log?: (message: string) => void;
  channel?: string;
  action?: "start" | "stop";
  target?: string;
  error?: unknown;
}): void {
  const target = params.target ? ` target=${params.target}` : "";
  const action = params.action ? ` action=${params.action}` : "";
  const message = `${params.channel ?? "channel"} typing${action} failed${target}: ${String(params.error)}`;
  params.log?.(message);
  params.logger?.warn?.(message);
}
