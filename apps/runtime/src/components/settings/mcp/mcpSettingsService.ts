import { invoke } from "@tauri-apps/api/core";

export interface McpServerRecord {
  id: string;
  name: string;
  command: string;
  args?: string[];
}

export interface McpFormState {
  name: string;
  command: string;
  args: string;
  env: string;
}

export const MCP_PRESETS = [
  { label: "— 快速选择 —", value: "", name: "", command: "", args: "", env: "" },
  { label: "Filesystem", value: "filesystem", name: "filesystem", command: "npx", args: "-y @anthropic/mcp-server-filesystem /tmp", env: "" },
  { label: "Brave Search", value: "brave-search", name: "brave-search", command: "npx", args: "-y @anthropic/mcp-server-brave-search", env: '{"BRAVE_API_KEY": ""}' },
  { label: "Memory", value: "memory", name: "memory", command: "npx", args: "-y @anthropic/mcp-server-memory", env: "" },
  { label: "Puppeteer", value: "puppeteer", name: "puppeteer", command: "npx", args: "-y @anthropic/mcp-server-puppeteer", env: "" },
  { label: "Fetch", value: "fetch", name: "fetch", command: "npx", args: "-y @anthropic/mcp-server-fetch", env: "" },
] as const;

export function parseMcpEnvJson(text: string): { env: Record<string, string>; error: string | null } {
  if (!text.trim()) {
    return { env: {}, error: null };
  }
  try {
    const parsed = JSON.parse(text) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return { env: {}, error: "环境变量 JSON 必须是对象格式" };
    }
    const normalized: Record<string, string> = {};
    for (const [key, value] of Object.entries(parsed as Record<string, unknown>)) {
      normalized[key] = typeof value === "string" ? value : String(value ?? "");
    }
    return { env: normalized, error: null };
  } catch {
    return { env: {}, error: "环境变量 JSON 格式错误" };
  }
}

export async function listMcpServers() {
  return invoke<McpServerRecord[]>("list_mcp_servers");
}

export async function addMcpServer(form: McpFormState) {
  const args = form.args.split(/\s+/).filter(Boolean);
  const parsedEnv = parseMcpEnvJson(form.env);
  if (parsedEnv.error) {
    throw new Error(parsedEnv.error);
  }

  await invoke("add_mcp_server", {
    name: form.name,
    command: form.command,
    args,
    env: parsedEnv.env,
  });
}

export async function removeMcpServer(id: string) {
  await invoke("remove_mcp_server", { id });
}
