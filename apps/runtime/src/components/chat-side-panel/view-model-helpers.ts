import type { Message, StreamItem, ToolCallInfo } from "../../types";
import { getToolResultDetailString } from "../../lib/tool-result";

export function getToolDisplayLabel(name: ToolCallInfo["name"]): string {
  switch (name) {
    case "write_file":
      return "写入文件";
    case "edit":
      return "编辑文件";
    case "web_search":
      return "资料搜索";
    case "todo_write":
      return "任务清单";
    default:
      return name;
  }
}

export function flattenToolCalls(messages: Message[]): ToolCallInfo[] {
  const streamToolCalls = messages.flatMap((message) =>
    (message.streamItems || [])
      .filter((item: StreamItem) => item.type === "tool_call" && item.toolCall)
      .map((item) => item.toolCall!),
  );

  const legacyToolCalls = messages.flatMap((message) => message.toolCalls || []);

  return [...streamToolCalls, ...legacyToolCalls];
}

export function normalizeTaskStatus(value: unknown): "pending" | "in_progress" | "completed" {
  if (value === "completed" || value === "in_progress") return value;
  return "pending";
}

export function readTouchedPath(tc: ToolCallInfo): string {
  const detailPath = getToolResultDetailString(tc.output, "path");
  if (detailPath) return detailPath;
  const input = tc.input || {};
  const candidate = input.path || input.file_path;
  return typeof candidate === "string" ? candidate : "";
}

export function classifyDeliverable(path: string): "primary" | "secondary" {
  const lowerPath = path.toLowerCase();
  if (
    lowerPath.endsWith(".docx") ||
    lowerPath.endsWith(".doc") ||
    lowerPath.endsWith(".pdf") ||
    lowerPath.endsWith(".html")
  ) {
    return "primary";
  }
  return "secondary";
}

export function inferCurrentTaskTitle(toolCalls: ToolCallInfo[]): string {
  const latestWrite = [...toolCalls]
    .reverse()
    .find((tc) => (tc.name === "write_file" || tc.name === "edit") && tc.status === "completed");
  if (latestWrite) {
    return "生成交付文件";
  }
  const latestSearch = [...toolCalls].reverse().find((tc) => tc.name === "web_search");
  if (latestSearch) {
    return "搜索资料";
  }
  return "";
}

export function buildRunningToolTitle(toolCall: ToolCallInfo | undefined): string {
  if (!toolCall) return "";
  if (toolCall.name === "bash") {
    const command = String(toolCall.input?.command || "").trim();
    if (!command) return "执行命令：bash";
    const compact = command.length > 48 ? `${command.slice(0, 48)}...` : command;
    return `执行命令：${compact}`;
  }
  if (toolCall.name === "web_search") {
    return "搜索资料";
  }
  if (toolCall.name === "write_file" || toolCall.name === "edit") {
    return "生成交付文件";
  }
  if (toolCall.name === "todo_write") {
    return "更新任务清单";
  }
  return `执行工具：${toolCall.name}`;
}

export function extractDomain(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return "";
  }
}

export function parseWebSearchResults(
  output: string | undefined,
): Array<{
  title: string;
  url: string;
  snippet: string;
  domain: string;
}> {
  if (!output) return [];

  try {
    const parsed = JSON.parse(output);
    const results = Array.isArray(parsed?.results)
      ? parsed.results
      : Array.isArray(parsed?.items)
        ? parsed.items
        : [];
    return results
      .map((item: any) => ({
        title: String(item?.title || ""),
        url: String(item?.url || item?.link || ""),
        snippet: String(item?.snippet || item?.summary || ""),
        domain: extractDomain(String(item?.url || item?.link || "")),
      }))
      .filter((item: { title: string; url: string }) => item.title || item.url);
  } catch {
    return output
      .split(/\n+/)
      .map((line: string) => line.trim())
      .filter(Boolean)
      .reduce<Array<{ title: string; url: string; snippet: string; domain: string }>>((acc, line: string) => {
        const match = line.match(/^(\d+)\.\s+(.*)$/);
        if (match) {
          acc.push({ title: match[2], url: "", snippet: "", domain: "" });
        } else if (acc.length > 0 && !acc[acc.length - 1].url && /^https?:\/\//.test(line)) {
          acc[acc.length - 1].url = line;
          acc[acc.length - 1].domain = extractDomain(line);
        } else if (acc.length > 0 && !acc[acc.length - 1].snippet) {
          acc[acc.length - 1].snippet = line;
        }
        return acc;
      }, []);
  }
}
