import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ModelConfig } from "../types";

const MCP_PRESETS = [
  { label: "— 快速选择 —", value: "", name: "", command: "", args: "", env: "" },
  { label: "Filesystem", value: "filesystem", name: "filesystem", command: "npx", args: "-y @anthropic/mcp-server-filesystem /tmp", env: "" },
  { label: "Brave Search", value: "brave-search", name: "brave-search", command: "npx", args: "-y @anthropic/mcp-server-brave-search", env: '{"BRAVE_API_KEY": ""}' },
  { label: "Memory", value: "memory", name: "memory", command: "npx", args: "-y @anthropic/mcp-server-memory", env: "" },
  { label: "Puppeteer", value: "puppeteer", name: "puppeteer", command: "npx", args: "-y @anthropic/mcp-server-puppeteer", env: "" },
  { label: "Fetch", value: "fetch", name: "fetch", command: "npx", args: "-y @anthropic/mcp-server-fetch", env: "" },
];

const PROVIDER_PRESETS = [
  { label: "— 快速选择 —", value: "", models: [] as string[] },
  { label: "OpenAI", value: "openai", api_format: "openai", base_url: "https://api.openai.com/v1", model_name: "gpt-4o-mini", models: ["gpt-4o", "gpt-4o-mini", "gpt-4.1", "gpt-4.1-mini", "gpt-4.1-nano", "o3-mini"] },
  { label: "Claude (Anthropic)", value: "anthropic", api_format: "anthropic", base_url: "https://api.anthropic.com/v1", model_name: "claude-3-5-haiku-20241022", models: ["claude-sonnet-4-5-20250929", "claude-3-5-haiku-20241022", "claude-3-5-sonnet-20241022"] },
  { label: "MiniMax (OpenAI 兼容)", value: "minimax-oai", api_format: "openai", base_url: "https://api.minimax.io/v1", model_name: "MiniMax-M2.5", models: ["MiniMax-M2.5", "MiniMax-M1", "MiniMax-Text-01"] },
  { label: "MiniMax (Anthropic 兼容)", value: "minimax-ant", api_format: "anthropic", base_url: "https://api.minimax.io/anthropic/v1", model_name: "MiniMax-M2.5", models: ["MiniMax-M2.5", "MiniMax-M1", "MiniMax-Text-01"] },
  { label: "DeepSeek", value: "deepseek", api_format: "openai", base_url: "https://api.deepseek.com/v1", model_name: "deepseek-chat", models: ["deepseek-chat", "deepseek-reasoner"] },
  { label: "Qwen (国际)", value: "qwen-intl", api_format: "openai", base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1", model_name: "qwen-max", models: ["qwen-max", "qwen-plus", "qwen-turbo", "qwen-long", "qwen-vl-max", "qwen-vl-plus"] },
  { label: "Qwen (国内)", value: "qwen-cn", api_format: "openai", base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1", model_name: "qwen-max", models: ["qwen-max", "qwen-plus", "qwen-turbo", "qwen-long", "qwen-vl-max", "qwen-vl-plus"] },
  { label: "Moonshot / Kimi", value: "moonshot", api_format: "openai", base_url: "https://api.moonshot.ai/v1", model_name: "kimi-k2", models: ["kimi-k2", "moonshot-v1-8k", "moonshot-v1-32k", "moonshot-v1-128k"] },
  { label: "Yi", value: "yi", api_format: "openai", base_url: "https://api.lingyiwanwu.com/v1", model_name: "yi-large", models: ["yi-large", "yi-medium", "yi-spark"] },
  { label: "自定义", value: "custom", models: [] as string[] },
];

const SEARCH_PRESETS = [
  { label: "— 快速选择 —", value: "", api_format: "", base_url: "", model_name: "" },
  { label: "Brave Search (国际首选)", value: "brave", api_format: "search_brave", base_url: "https://api.search.brave.com", model_name: "" },
  { label: "Tavily (AI 专用)", value: "tavily", api_format: "search_tavily", base_url: "https://api.tavily.com", model_name: "" },
  { label: "秘塔搜索 (中文首选)", value: "metaso", api_format: "search_metaso", base_url: "https://metaso.cn", model_name: "" },
  { label: "博查搜索 (中文 AI)", value: "bocha", api_format: "search_bocha", base_url: "https://api.bochaai.com", model_name: "" },
  { label: "SerpAPI (多引擎)", value: "serpapi", api_format: "search_serpapi", base_url: "https://serpapi.com", model_name: "google" },
];

interface Props {
  onClose: () => void;
}

interface RoutingSettings {
  max_call_depth: number;
  node_timeout_seconds: number;
  retry_count: number;
}

export function SettingsView({ onClose }: Props) {
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [form, setForm] = useState({
    name: "",
    api_format: "openai",
    base_url: "https://api.openai.com/v1",
    model_name: "gpt-4o-mini",
    api_key: "",
  });
  const [error, setError] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [modelSuggestions, setModelSuggestions] = useState<string[]>([]);

  // 编辑状态 + API Key 可见性
  const [editingModelId, setEditingModelId] = useState<string | null>(null);
  const [showApiKey, setShowApiKey] = useState(false);

  // MCP 服务器管理
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [mcpServers, setMcpServers] = useState<any[]>([]);
  const [mcpForm, setMcpForm] = useState({ name: "", command: "", args: "", env: "" });
  const [mcpError, setMcpError] = useState("");
  const [activeTab, setActiveTab] = useState<"models" | "mcp" | "search" | "routing">("models");

  // 搜索引擎配置
  const [searchConfigs, setSearchConfigs] = useState<ModelConfig[]>([]);
  const [searchForm, setSearchForm] = useState({ name: "", api_format: "", base_url: "", model_name: "", api_key: "" });
  const [searchError, setSearchError] = useState("");
  const [searchTesting, setSearchTesting] = useState(false);
  const [searchTestResult, setSearchTestResult] = useState<boolean | null>(null);

  // 搜索引擎编辑状态 + API Key 可见性
  const [editingSearchId, setEditingSearchId] = useState<string | null>(null);
  const [showSearchApiKey, setShowSearchApiKey] = useState(false);
  const [routeSettings, setRouteSettings] = useState<RoutingSettings>({
    max_call_depth: 4,
    node_timeout_seconds: 60,
    retry_count: 0,
  });
  const [routeSaveState, setRouteSaveState] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [routeError, setRouteError] = useState("");

  useEffect(() => { loadModels(); loadMcpServers(); loadSearchConfigs(); loadRoutingSettings(); }, []);

  async function loadModels() {
    const list = await invoke<ModelConfig[]>("list_model_configs");
    setModels(list);
  }

  async function loadSearchConfigs() {
    try {
      const list = await invoke<ModelConfig[]>("list_search_configs");
      setSearchConfigs(list);
    } catch (e) {
      console.error("加载搜索配置失败:", e);
    }
  }

  async function loadRoutingSettings() {
    try {
      const settings = await invoke<RoutingSettings>("get_routing_settings");
      setRouteSettings(settings);
    } catch (e) {
      setRouteError("加载自动路由设置失败: " + String(e));
      setRouteSaveState("error");
    }
  }

  async function handleSaveRoutingSettings() {
    setRouteSaveState("saving");
    setRouteError("");
    try {
      await invoke("set_routing_settings", {
        settings: {
          max_call_depth: Math.max(2, Math.min(8, routeSettings.max_call_depth)),
          node_timeout_seconds: Math.max(5, Math.min(600, routeSettings.node_timeout_seconds)),
          retry_count: Math.max(0, Math.min(2, routeSettings.retry_count)),
        },
      });
      setRouteSaveState("saved");
      setTimeout(() => setRouteSaveState("idle"), 1200);
    } catch (e) {
      setRouteError("保存自动路由设置失败: " + String(e));
      setRouteSaveState("error");
    }
  }

  // 加载已保存的模型配置到表单（用于编辑）
  async function handleEditModel(m: ModelConfig) {
    try {
      const apiKey = await invoke<string>("get_model_api_key", { modelId: m.id });
      setForm({
        name: m.name,
        api_format: m.api_format,
        base_url: m.base_url,
        model_name: m.model_name,
        api_key: apiKey,
      });
      setEditingModelId(m.id);
      setShowApiKey(false);
      setError("");
      setTestResult(null);
      // 更新模型建议列表
      const preset = PROVIDER_PRESETS.find((p) => p.api_format === m.api_format && p.base_url === m.base_url);
      setModelSuggestions(preset?.models || []);
    } catch (e) {
      setError("加载配置失败: " + String(e));
    }
  }

  // 加载已保存的搜索配置到表单（用于编辑）
  async function handleEditSearch(s: ModelConfig) {
    try {
      const apiKey = await invoke<string>("get_model_api_key", { modelId: s.id });
      setSearchForm({
        name: s.name,
        api_format: s.api_format,
        base_url: s.base_url,
        model_name: s.model_name,
        api_key: apiKey,
      });
      setEditingSearchId(s.id);
      setShowSearchApiKey(false);
      setSearchError("");
      setSearchTestResult(null);
    } catch (e) {
      setSearchError("加载配置失败: " + String(e));
    }
  }

  async function handleSave() {
    setError("");
    try {
      await invoke("save_model_config", {
        config: {
          id: editingModelId || "",
          name: form.name,
          api_format: form.api_format,
          base_url: form.base_url,
          model_name: form.model_name,
          is_default: editingModelId
            ? models.find((m) => m.id === editingModelId)?.is_default ?? false
            : models.length === 0,
        },
        apiKey: form.api_key,
      });
      setForm({ name: "", api_format: "openai", base_url: "https://api.openai.com/v1", model_name: "gpt-4o-mini", api_key: "" });
      setEditingModelId(null);
      setShowApiKey(false);
      loadModels();
    } catch (e: unknown) {
      setError(String(e));
    }
  }

  async function handleTest() {
    setTesting(true);
    setTestResult(null);
    try {
      const ok = await invoke<boolean>("test_connection_cmd", {
        config: {
          id: "",
          name: form.name,
          api_format: form.api_format,
          base_url: form.base_url,
          model_name: form.model_name,
          is_default: false,
        },
        apiKey: form.api_key,
      });
      setTestResult(ok);
    } catch (e: unknown) {
      setError(String(e));
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  }

  function applyPreset(value: string) {
    const preset = PROVIDER_PRESETS.find((p) => p.value === value);
    if (!preset || !preset.api_format) {
      setModelSuggestions([]);
      return;
    }
    setForm((f) => ({
      ...f,
      api_format: preset.api_format!,
      base_url: preset.base_url!,
      model_name: preset.model_name!,
    }));
    setModelSuggestions(preset.models);
  }

  function applyMcpPreset(value: string) {
    const preset = MCP_PRESETS.find((p) => p.value === value);
    if (!preset || !preset.value) return;
    setMcpForm({
      name: preset.name,
      command: preset.command,
      args: preset.args,
      env: preset.env,
    });
  }

  function applySearchPreset(value: string) {
    const preset = SEARCH_PRESETS.find((p) => p.value === value);
    if (!preset || !preset.value) return;
    setSearchForm((f) => ({
      ...f,
      name: preset.label.replace(/ \(.*\)/, ""),
      api_format: preset.api_format,
      base_url: preset.base_url,
      model_name: preset.model_name,
    }));
  }

  async function handleDelete(id: string) {
    await invoke("delete_model_config", { modelId: id });
    // 若删除的是当前编辑项，重置表单
    if (editingModelId === id) {
      setEditingModelId(null);
      setShowApiKey(false);
      setForm({ name: "", api_format: "openai", base_url: "https://api.openai.com/v1", model_name: "gpt-4o-mini", api_key: "" });
      setError("");
      setTestResult(null);
    }
    loadModels();
  }

  async function loadMcpServers() {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const list = await invoke<any[]>("list_mcp_servers");
      setMcpServers(list);
    } catch (e) {
      console.error("加载 MCP 服务器失败:", e);
    }
  }

  async function handleAddMcp() {
    setMcpError("");
    try {
      const args = mcpForm.args.split(/\s+/).filter(Boolean);
      let env: Record<string, string> = {};
      if (mcpForm.env.trim()) {
        try {
          env = JSON.parse(mcpForm.env.trim());
        } catch {
          setMcpError("环境变量 JSON 格式错误");
          return;
        }
      }
      await invoke("add_mcp_server", {
        name: mcpForm.name,
        command: mcpForm.command,
        args,
        env,
      });
      setMcpForm({ name: "", command: "", args: "", env: "" });
      loadMcpServers();
    } catch (e) {
      setMcpError(String(e));
    }
  }

  async function handleRemoveMcp(id: string) {
    await invoke("remove_mcp_server", { id });
    loadMcpServers();
  }

  async function handleSaveSearch() {
    setSearchError("");
    try {
      await invoke("save_model_config", {
        config: {
          id: editingSearchId || "",
          name: searchForm.name,
          api_format: searchForm.api_format,
          base_url: searchForm.base_url,
          model_name: searchForm.model_name,
          is_default: editingSearchId
            ? searchConfigs.find((s) => s.id === editingSearchId)?.is_default ?? false
            : searchConfigs.length === 0,
        },
        apiKey: searchForm.api_key,
      });
      setSearchForm({ name: "", api_format: "", base_url: "", model_name: "", api_key: "" });
      setEditingSearchId(null);
      setShowSearchApiKey(false);
      loadSearchConfigs();
    } catch (e) {
      setSearchError(String(e));
    }
  }

  async function handleTestSearch() {
    setSearchTesting(true);
    setSearchTestResult(null);
    try {
      const ok = await invoke<boolean>("test_search_connection", {
        config: {
          id: "",
          name: searchForm.name,
          api_format: searchForm.api_format,
          base_url: searchForm.base_url,
          model_name: searchForm.model_name,
          is_default: false,
        },
        apiKey: searchForm.api_key,
      });
      setSearchTestResult(ok);
    } catch (e) {
      setSearchError(String(e));
      setSearchTestResult(false);
    } finally {
      setSearchTesting(false);
    }
  }

  async function handleSetDefaultSearch(id: string) {
    await invoke("set_default_search", { configId: id });
    loadSearchConfigs();
  }

  async function handleDeleteSearch(id: string) {
    await invoke("delete_model_config", { modelId: id });
    // 若删除的是当前编辑项，重置表单
    if (editingSearchId === id) {
      setEditingSearchId(null);
      setShowSearchApiKey(false);
      setSearchForm({ name: "", api_format: "", base_url: "", model_name: "", api_key: "" });
      setSearchError("");
      setSearchTestResult(null);
    }
    loadSearchConfigs();
  }

  const inputCls = "w-full bg-gray-50 border border-gray-200 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400";
  const labelCls = "block text-xs text-gray-500 mb-1";

  // 眼睛图标：显示状态（可见）
  function EyeOpenIcon() {
    return (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
        <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    );
  }

  // 眼睛图标：隐藏状态（划线）
  function EyeSlashIcon() {
    return (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
      </svg>
    );
  }

  return (
    <div className="flex flex-col h-full p-6 overflow-y-auto">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <button
            onClick={() => setActiveTab("models")}
            className={"text-sm font-medium pb-1 border-b-2 transition-colors " +
              (activeTab === "models" ? "text-gray-800 border-blue-500" : "text-gray-500 border-transparent hover:text-gray-700")}
          >
            模型配置
          </button>
          <button
            onClick={() => setActiveTab("mcp")}
            className={"text-sm font-medium pb-1 border-b-2 transition-colors " +
              (activeTab === "mcp" ? "text-gray-800 border-blue-500" : "text-gray-500 border-transparent hover:text-gray-700")}
          >
            MCP 服务器
          </button>
          <button
            onClick={() => setActiveTab("search")}
            className={"text-sm font-medium pb-1 border-b-2 transition-colors " +
              (activeTab === "search" ? "text-gray-800 border-blue-500" : "text-gray-500 border-transparent hover:text-gray-700")}
          >
            搜索引擎
          </button>
          <button
            onClick={() => setActiveTab("routing")}
            className={"text-sm font-medium pb-1 border-b-2 transition-colors " +
              (activeTab === "routing" ? "text-gray-800 border-blue-500" : "text-gray-500 border-transparent hover:text-gray-700")}
          >
            自动路由
          </button>
        </div>
        <button onClick={onClose} className="text-gray-500 hover:text-gray-800 text-sm">
          返回
        </button>
      </div>

      {activeTab === "models" && (<>
      {models.length > 0 && (
        <div className="mb-6 space-y-2">
          <div className="text-xs text-gray-500 mb-2">已配置模型</div>
          {models.map((m) => (
            <div
              key={m.id}
              className={
                "flex items-center justify-between bg-white rounded-lg px-4 py-2.5 text-sm border transition-colors " +
                (editingModelId === m.id ? "border-blue-400 ring-1 ring-blue-400" : "border-transparent hover:border-gray-200")
              }
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-gray-800">{m.name}</span>
                  {m.is_default && (
                    <span className="text-[10px] bg-blue-500 text-white px-1.5 py-0.5 rounded">默认</span>
                  )}
                </div>
                <div className="text-xs text-gray-400 mt-0.5 truncate">
                  {m.model_name} · {m.api_format === "anthropic" ? "Anthropic" : "OpenAI 兼容"} · {m.base_url}
                </div>
              </div>
              <div className="flex items-center gap-2 flex-shrink-0 ml-3">
                <button
                  onClick={() => handleEditModel(m)}
                  className="text-blue-500 hover:text-blue-600 text-xs"
                >
                  编辑
                </button>
                <button
                  onClick={() => handleDelete(m.id)}
                  className="text-red-400 hover:text-red-500 text-xs"
                >
                  删除
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="bg-white rounded-lg p-4 space-y-3">
        <div className="flex items-center justify-between mb-2">
          <div className="text-xs font-medium text-gray-500">
            {editingModelId ? "编辑模型" : "添加模型"}
          </div>
          {editingModelId && (
            <button
              onClick={() => {
                setEditingModelId(null);
                setShowApiKey(false);
                setForm({ name: "", api_format: "openai", base_url: "https://api.openai.com/v1", model_name: "gpt-4o-mini", api_key: "" });
                setError("");
                setTestResult(null);
              }}
              className="text-xs text-gray-400 hover:text-gray-600"
            >
              取消编辑
            </button>
          )}
        </div>
        <div>
          <label className={labelCls}>快速选择 Provider</label>
          <select
            className={inputCls}
            defaultValue=""
            onChange={(e) => applyPreset(e.target.value)}
          >
            {PROVIDER_PRESETS.map((p) => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
        </div>
        <div>
          <label className={labelCls}>名称</label>
          <input className={inputCls} value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>API 格式</label>
          <select className={inputCls} value={form.api_format} onChange={(e) => setForm({ ...form, api_format: e.target.value })}>
            <option value="openai">OpenAI 兼容</option>
            <option value="anthropic">Anthropic (Claude)</option>
          </select>
        </div>
        <div>
          <label className={labelCls}>Base URL</label>
          <input className={inputCls} value={form.base_url} onChange={(e) => setForm({ ...form, base_url: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>模型名称</label>
          <input className={inputCls} list="model-suggestions" value={form.model_name} onChange={(e) => setForm({ ...form, model_name: e.target.value })} />
          {modelSuggestions.length > 0 && (
            <datalist id="model-suggestions">
              {modelSuggestions.map((m) => (
                <option key={m} value={m} />
              ))}
            </datalist>
          )}
        </div>
        <div>
          <label className={labelCls}>API Key</label>
          <div className="relative">
            <input
              className={inputCls + " pr-10"}
              type={showApiKey ? "text" : "password"}
              value={form.api_key}
              onChange={(e) => setForm({ ...form, api_key: e.target.value })}
            />
            <button
              type="button"
              onClick={() => setShowApiKey(!showApiKey)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 p-1"
              title={showApiKey ? "隐藏" : "显示"}
            >
              {showApiKey ? <EyeSlashIcon /> : <EyeOpenIcon />}
            </button>
          </div>
        </div>
        {error && <div className="bg-red-50 text-red-600 text-xs px-2 py-1 rounded">{error}</div>}
        {testResult !== null && (
          <div className={"text-xs " + (testResult ? "bg-green-50 text-green-600 px-2 py-1 rounded" : "bg-red-50 text-red-600 px-2 py-1 rounded")}>
            {testResult ? "连接成功" : "连接失败，请检查配置"}
          </div>
        )}
        <div className="flex gap-2 pt-1">
          <button
            onClick={handleTest}
            disabled={testing}
            className="flex-1 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
          >
            {testing ? "测试中..." : "测试连接"}
          </button>
          <button
            onClick={handleSave}
            className="flex-1 bg-blue-500 hover:bg-blue-600 text-white text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
          >
            {editingModelId ? "保存修改" : "保存"}
          </button>
        </div>
      </div>
      </>)}

      {activeTab === "mcp" && (<>
      {/* MCP 服务器管理 */}
      <div className="bg-white rounded-lg p-4 space-y-3">
        <div className="text-xs font-medium text-gray-500 mb-2">MCP 服务器</div>

        {mcpServers.length > 0 && (
          <div className="space-y-2 mb-3">
            {mcpServers.map((s) => (
              <div key={s.id} className="flex items-center justify-between bg-gray-100 rounded px-3 py-2 text-sm">
                <div>
                  <span className="font-medium">{s.name}</span>
                  <span className="text-gray-500 ml-2 text-xs">{s.command} {s.args?.join(" ")}</span>
                </div>
                <button onClick={() => handleRemoveMcp(s.id)} className="text-red-400 hover:text-red-300 text-xs">
                  删除
                </button>
              </div>
            ))}
          </div>
        )}

        <div>
          <label className={labelCls}>快速选择 MCP 服务器</label>
          <select
            className={inputCls}
            defaultValue=""
            onChange={(e) => applyMcpPreset(e.target.value)}
          >
            {MCP_PRESETS.map((p) => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
        </div>
        <div>
          <label className={labelCls}>名称</label>
          <input className={inputCls} placeholder="例: filesystem" value={mcpForm.name} onChange={(e) => setMcpForm({ ...mcpForm, name: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>命令</label>
          <input className={inputCls} placeholder="例: npx" value={mcpForm.command} onChange={(e) => setMcpForm({ ...mcpForm, command: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>参数（空格分隔）</label>
          <input className={inputCls} placeholder="例: @anthropic/mcp-server-filesystem /tmp" value={mcpForm.args} onChange={(e) => setMcpForm({ ...mcpForm, args: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>环境变量（JSON 格式，可选）</label>
          <input className={inputCls} placeholder='例: {"API_KEY": "xxx"}' value={mcpForm.env} onChange={(e) => setMcpForm({ ...mcpForm, env: e.target.value })} />
        </div>
        {mcpError && <div className="bg-red-50 text-red-600 text-xs px-2 py-1 rounded">{mcpError}</div>}
        <button
          onClick={handleAddMcp}
          disabled={!mcpForm.name || !mcpForm.command}
          className="w-full bg-blue-500 hover:bg-blue-600 disabled:bg-gray-200 disabled:text-gray-400 text-white text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
        >
          添加 MCP 服务器
        </button>
      </div>
      </>)}

      {activeTab === "search" && (<>
        {searchConfigs.length > 0 && (
          <div className="mb-6 space-y-2">
            <div className="text-xs text-gray-500 mb-2">已配置搜索引擎</div>
            {searchConfigs.map((s) => (
              <div
                key={s.id}
                className={
                  "flex items-center justify-between bg-white rounded-lg px-4 py-2.5 text-sm border transition-colors " +
                  (editingSearchId === s.id ? "border-blue-400 ring-1 ring-blue-400" : "border-transparent hover:border-gray-200")
                }
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-gray-800">{s.name}</span>
                    {s.is_default && (
                      <span className="text-[10px] bg-blue-500 text-white px-1.5 py-0.5 rounded">默认</span>
                    )}
                  </div>
                  <div className="text-xs text-gray-400 mt-0.5 truncate">
                    {s.api_format.replace("search_", "")} · {s.base_url}
                  </div>
                </div>
                <div className="flex items-center gap-2 flex-shrink-0 ml-3">
                  {!s.is_default && (
                    <button onClick={() => handleSetDefaultSearch(s.id)} className="text-blue-400 hover:text-blue-500 text-xs">
                      设为默认
                    </button>
                  )}
                  <button onClick={() => handleEditSearch(s)} className="text-blue-500 hover:text-blue-600 text-xs">
                    编辑
                  </button>
                  <button onClick={() => handleDeleteSearch(s.id)} className="text-red-400 hover:text-red-500 text-xs">
                    删除
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        <div className="bg-white rounded-lg p-4 space-y-3">
          <div className="flex items-center justify-between mb-2">
            <div className="text-xs font-medium text-gray-500">
              {editingSearchId ? "编辑搜索引擎" : "添加搜索引擎"}
            </div>
            {editingSearchId && (
              <button
                onClick={() => {
                  setEditingSearchId(null);
                  setShowSearchApiKey(false);
                  setSearchForm({ name: "", api_format: "", base_url: "", model_name: "", api_key: "" });
                  setSearchError("");
                  setSearchTestResult(null);
                }}
                className="text-xs text-gray-400 hover:text-gray-600"
              >
                取消编辑
              </button>
            )}
          </div>
          <div>
            <label className={labelCls}>快速选择搜索引擎</label>
            <select className={inputCls} defaultValue="" onChange={(e) => applySearchPreset(e.target.value)}>
              {SEARCH_PRESETS.map((p) => (
                <option key={p.value} value={p.value}>{p.label}</option>
              ))}
            </select>
          </div>
          <div>
            <label className={labelCls}>名称</label>
            <input className={inputCls} value={searchForm.name} onChange={(e) => setSearchForm({ ...searchForm, name: e.target.value })} />
          </div>
          <div>
            <label className={labelCls}>API Key</label>
            <div className="relative">
              <input
                className={inputCls + " pr-10"}
                type={showSearchApiKey ? "text" : "password"}
                value={searchForm.api_key}
                onChange={(e) => setSearchForm({ ...searchForm, api_key: e.target.value })}
              />
              <button
                type="button"
                onClick={() => setShowSearchApiKey(!showSearchApiKey)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 p-1"
                title={showSearchApiKey ? "隐藏" : "显示"}
              >
                {showSearchApiKey ? <EyeSlashIcon /> : <EyeOpenIcon />}
              </button>
            </div>
          </div>
          <div>
            <label className={labelCls}>Base URL（可选自定义）</label>
            <input className={inputCls} value={searchForm.base_url} onChange={(e) => setSearchForm({ ...searchForm, base_url: e.target.value })} />
          </div>
          {searchForm.api_format === "search_serpapi" && (
            <div>
              <label className={labelCls}>搜索引擎 (google/baidu/bing)</label>
              <input className={inputCls} value={searchForm.model_name} onChange={(e) => setSearchForm({ ...searchForm, model_name: e.target.value })} />
            </div>
          )}
          {searchError && <div className="bg-red-50 text-red-600 text-xs px-2 py-1 rounded">{searchError}</div>}
          {searchTestResult !== null && (
            <div className={"text-xs px-2 py-1 rounded " + (searchTestResult ? "bg-green-50 text-green-600" : "bg-red-50 text-red-600")}>
              {searchTestResult ? "连接成功" : "连接失败，请检查配置"}
            </div>
          )}
          <div className="flex gap-2 pt-1">
            <button
              onClick={handleTestSearch}
              disabled={searchTesting || !searchForm.api_format}
              className="flex-1 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
            >
              {searchTesting ? "测试中..." : "测试连接"}
            </button>
            <button
              onClick={handleSaveSearch}
              disabled={!searchForm.name || !searchForm.api_format || !searchForm.api_key}
              className="flex-1 bg-blue-500 hover:bg-blue-600 disabled:opacity-50 text-white text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
            >
              {editingSearchId ? "保存修改" : "保存"}
            </button>
          </div>
        </div>
      </>)}

      {activeTab === "routing" && (
        <div className="bg-white rounded-lg p-4 space-y-3">
          <div className="text-xs font-medium text-gray-500 mb-2">子 Skill 自动路由</div>
          <div>
            <label className={labelCls}>最大调用深度 (2-8)</label>
            <input
              className={inputCls}
              type="number"
              min={2}
              max={8}
              value={routeSettings.max_call_depth}
              onChange={(e) => setRouteSettings((s) => ({ ...s, max_call_depth: Number(e.target.value || 4) }))}
            />
          </div>
          <div>
            <label className={labelCls}>节点超时秒数 (5-600)</label>
            <input
              className={inputCls}
              type="number"
              min={5}
              max={600}
              value={routeSettings.node_timeout_seconds}
              onChange={(e) => setRouteSettings((s) => ({ ...s, node_timeout_seconds: Number(e.target.value || 60) }))}
            />
          </div>
          <div>
            <label className={labelCls}>失败重试次数 (0-2)</label>
            <input
              className={inputCls}
              type="number"
              min={0}
              max={2}
              value={routeSettings.retry_count}
              onChange={(e) => setRouteSettings((s) => ({ ...s, retry_count: Number(e.target.value || 0) }))}
            />
          </div>
          {routeError && <div className="bg-red-50 text-red-600 text-xs px-2 py-1 rounded">{routeError}</div>}
          {routeSaveState === "saved" && (
            <div className="bg-green-50 text-green-600 text-xs px-2 py-1 rounded">已保存</div>
          )}
          <button
            onClick={handleSaveRoutingSettings}
            disabled={routeSaveState === "saving"}
            className="w-full bg-blue-500 hover:bg-blue-600 disabled:opacity-50 text-white text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
          >
            {routeSaveState === "saving" ? "保存中..." : "保存自动路由设置"}
          </button>
        </div>
      )}
    </div>
  );
}
