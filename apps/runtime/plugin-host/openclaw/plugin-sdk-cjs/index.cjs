"use strict";

const DEFAULT_ACCOUNT_ID = "default";
const DEFAULT_GROUP_HISTORY_LIMIT = 50;
const PAIRING_APPROVED_MESSAGE =
  "Pairing approved. You can now message this bot directly.";
const SILENT_REPLY_TOKEN = "NO_REPLY";

function emptyPluginConfigSchema() {
  return {
    type: "object",
    additionalProperties: false,
    properties: {},
  };
}

function normalizeAccountId(value) {
  const normalized = typeof value === "string" ? value.trim() : "";
  return normalized || DEFAULT_ACCOUNT_ID;
}

function buildRandomTempFilePath(params = {}) {
  const prefix = typeof params.prefix === "string" && params.prefix.trim()
    ? params.prefix.trim()
    : "workclaw-plugin";
  const fileName =
    typeof params.fileName === "string" && params.fileName.trim() ? params.fileName.trim() : "";
  const extension =
    typeof params.extension === "string" && params.extension.trim() ? params.extension.trim() : "";
  const suffix = `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  if (fileName) {
    return `${prefix}-${suffix}-${fileName}`;
  }
  return `${prefix}-${suffix}${extension}`;
}

function addWildcardAllowFrom(allowFrom) {
  const normalized = (allowFrom ?? [])
    .map((entry) => String(entry).trim())
    .filter(Boolean);
  return normalized.includes("*") ? normalized : [...normalized, "*"];
}

function formatDocsLink(path) {
  const normalized = path.startsWith("/") ? path : `/${path}`;
  return `https://docs.openclaw.ai${normalized}`;
}

function buildPendingHistoryContextFromMap(params) {
  const historyMap = params?.historyMap;
  const historyKey = typeof params?.historyKey === "string" ? params.historyKey.trim() : "";
  const limit = typeof params?.limit === "number" ? params.limit : 0;
  const currentMessage = String(params?.currentMessage ?? "");
  if (!historyMap || !historyKey || limit <= 0) {
    return currentMessage;
  }
  const entries = (historyMap.get(historyKey) ?? []).slice(-limit);
  if (entries.length === 0) {
    return currentMessage;
  }
  const formatter =
    typeof params?.formatEntry === "function"
      ? params.formatEntry
      : (entry) => `${String(entry?.sender ?? "user")}: ${String(entry?.body ?? "")}`;
  const lineBreak = typeof params?.lineBreak === "string" ? params.lineBreak : "\n";
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

function resolveThreadSessionKeys(params) {
  const baseSessionKey = String(params?.baseSessionKey ?? "");
  const threadId = typeof params?.threadId === "string" ? params.threadId.trim() : "";
  if (!threadId) {
    return {
      sessionKey: baseSessionKey,
      parentSessionKey: params?.parentSessionKey,
    };
  }
  const normalizer =
    typeof params?.normalizeThreadId === "function"
      ? params.normalizeThreadId
      : (value) => value.toLowerCase();
  const normalizedThreadId = normalizer(threadId);
  const useSuffix = params?.useSuffix ?? true;
  return {
    sessionKey: useSuffix ? `${baseSessionKey}:thread:${normalizedThreadId}` : baseSessionKey,
    parentSessionKey: params?.parentSessionKey,
  };
}

function clearHistoryEntriesIfEnabled(params) {
  if (!params?.historyMap || !params?.historyKey || (params?.limit ?? 0) <= 0) {
    return;
  }
  params.historyMap.set(params.historyKey, []);
}

function recordPendingHistoryEntryIfEnabled(params) {
  if (!params?.historyMap || !params?.historyKey || !params?.entry || (params?.limit ?? 0) <= 0) {
    return [];
  }
  const history = params.historyMap.get(params.historyKey) ?? [];
  history.push(params.entry);
  while (history.length > params.limit) {
    history.shift();
  }
  params.historyMap.set(params.historyKey, history);
  return history;
}

async function resolveSenderCommandAuthorization(params = {}) {
  const cfg = params.cfg ?? {};
  const rawBody = String(params.rawBody ?? "");
  const shouldComputeAuth =
    typeof params.shouldComputeCommandAuthorized === "function"
      ? params.shouldComputeCommandAuthorized(rawBody, cfg)
      : rawBody.trim().startsWith("/");
  const configuredAllowFrom = params.configuredAllowFrom ?? [];
  const configuredGroupAllowFrom = params.configuredGroupAllowFrom ?? [];
  const storeAllowFrom =
    typeof params.readAllowFromStore === "function"
      ? await params.readAllowFromStore().catch(() => [])
      : [];
  const effectiveAllowFrom = Array.from(new Set([...configuredAllowFrom, ...storeAllowFrom]));
  const effectiveGroupAllowFrom = Array.from(new Set(configuredGroupAllowFrom));
  const senderId = String(params.senderId ?? "");
  const isSenderAllowed =
    typeof params.isSenderAllowed === "function"
      ? params.isSenderAllowed
      : (candidate, allowFrom) =>
          allowFrom.map((entry) => entry.trim().toLowerCase()).includes(candidate.trim().toLowerCase());
  const senderAllowedForCommands = isSenderAllowed(
    senderId,
    params.isGroup ? effectiveGroupAllowFrom : effectiveAllowFrom,
  );
  const commandAuthorized = shouldComputeAuth
    ? typeof params.resolveCommandAuthorizedFromAuthorizers === "function"
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

function isNormalizedSenderAllowed(params) {
  const normalizedSenderId =
    typeof params?.normalizedSenderId === "string" ? params.normalizedSenderId.trim().toLowerCase() : "";
  if (!normalizedSenderId) {
    return false;
  }
  const allowFrom = (params?.allowFrom ?? []).map((entry) =>
    String(entry).trim().toLowerCase(),
  );
  return allowFrom.includes("*") || allowFrom.includes(normalizedSenderId);
}

function extractToolSend(result) {
  return result;
}

function jsonResult(payload) {
  return { ok: true, payload };
}

function readStringParam(params, key) {
  const value = params?.[key];
  return typeof value === "string" ? value : undefined;
}

function readReactionParams(params) {
  return params;
}

function createReplyPrefixContext(params) {
  const prefix = typeof params?.prefix === "string" ? params.prefix.trim() : "";
  const text = params?.text ?? "";
  const prefixContext = {
    agentId: params?.agentId ?? "main",
    channel: params?.channel,
    accountId: params?.accountId,
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

function createTypingCallbacks(params = {}) {
  return {
    async onReplyStart() {
      if (typeof params.start !== "function") {
        return;
      }
      try {
        await params.start();
      } catch (error) {
        params.onStartError?.(error);
      }
    },
    onIdle() {
      if (typeof params.stop !== "function") {
        return;
      }
      void params.stop().catch((error) => params.onStopError?.(error));
    },
    onCleanup() {
      if (typeof params.stop !== "function") {
        return;
      }
      void params.stop().catch((error) => params.onStopError?.(error));
    },
  };
}

function logTypingFailure(params) {
  const target = params?.target ? ` target=${params.target}` : "";
  const action = params?.action ? ` action=${params.action}` : "";
  const message = `${params?.channel ?? "channel"} typing${action} failed${target}: ${String(params?.error)}`;
  params?.log?.(message);
  params?.logger?.warn?.(message);
}

module.exports = {
  DEFAULT_ACCOUNT_ID,
  DEFAULT_GROUP_HISTORY_LIMIT,
  PAIRING_APPROVED_MESSAGE,
  SILENT_REPLY_TOKEN,
  emptyPluginConfigSchema,
  normalizeAccountId,
  buildRandomTempFilePath,
  addWildcardAllowFrom,
  formatDocsLink,
  buildPendingHistoryContextFromMap,
  resolveThreadSessionKeys,
  clearHistoryEntriesIfEnabled,
  recordPendingHistoryEntryIfEnabled,
  resolveSenderCommandAuthorization,
  isNormalizedSenderAllowed,
  extractToolSend,
  jsonResult,
  readStringParam,
  readReactionParams,
  createReplyPrefixContext,
  createTypingCallbacks,
  logTypingFailure,
};
