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

  // MCP 服务器管理
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [mcpServers, setMcpServers] = useState<any[]>([]);
  const [mcpForm, setMcpForm] = useState({ name: "", command: "", args: "", env: "" });
  const [mcpError, setMcpError] = useState("");
  const [activeTab, setActiveTab] = useState<"models" | "mcp" | "search">("models");

  // 搜索引擎配置
  const [searchConfigs, setSearchConfigs] = useState<ModelConfig[]>([]);
  const [searchForm, setSearchForm] = useState({ name: "", api_format: "", base_url: "", model_name: "", api_key: "" });
  const [searchError, setSearchError] = useState("");
  const [searchTesting, setSearchTesting] = useState(false);
  const [searchTestResult, setSearchTestResult] = useState<boolean | null>(null);

  useEffect(() => { loadModels(); loadMcpServers(); loadSearchConfigs(); }, []);

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

  async function handleSave() {
    setError("");
    try {
      await invoke("save_model_config", {
        config: {
          id: "",
          name: form.name,
          api_format: form.api_format,
          base_url: form.base_url,
          model_name: form.model_name,
          is_default: models.length === 0,
        },
        apiKey: form.api_key,
      });
      setForm({ name: "", api_format: "openai", base_url: "https://api.openai.com/v1", model_name: "gpt-4o-mini", api_key: "" });
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
          id: "",
          name: searchForm.name,
          api_format: searchForm.api_format,
          base_url: searchForm.base_url,
          model_name: searchForm.model_name,
          is_default: searchConfigs.length === 0,
        },
        apiKey: searchForm.api_key,
      });
      setSearchForm({ name: "", api_format: "", base_url: "", model_name: "", api_key: "" });
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
    loadSearchConfigs();
  }

  const inputCls = "w-full bg-gray-50 border border-gray-200 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400";
  const labelCls = "block text-xs text-gray-500 mb-1";

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
            <div key={m.id} className="flex items-center justify-between bg-white rounded px-3 py-2 text-sm">
              <div>
                <span className="font-medium">{m.name}</span>
                <span className="text-gray-500 ml-2">{m.model_name}</span>
              </div>
              <button onClick={() => handleDelete(m.id)} className="text-red-400 hover:text-red-300 text-xs">
                删除
              </button>
            </div>
          ))}
        </div>
      )}

      <div className="bg-white rounded-lg p-4 space-y-3">
        <div className="text-xs font-medium text-gray-500 mb-2">添加模型</div>
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
          <input className={inputCls} type="password" value={form.api_key} onChange={(e) => setForm({ ...form, api_key: e.target.value })} />
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
            className="flex-1 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 text-sm py-1.5 rounded transition-colors"
          >
            {testing ? "测试中..." : "测试连接"}
          </button>
          <button
            onClick={handleSave}
            className="flex-1 bg-blue-500 hover:bg-blue-600 text-white text-sm py-1.5 rounded transition-colors"
          >
            保存
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
          className="w-full bg-blue-500 hover:bg-blue-600 disabled:bg-gray-200 disabled:text-gray-400 text-white text-sm py-1.5 rounded transition-colors"
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
              <div key={s.id} className="flex items-center justify-between bg-white rounded px-3 py-2 text-sm">
                <div className="flex items-center gap-2">
                  <span className="font-medium">{s.name}</span>
                  <span className="text-gray-500 text-xs">{s.api_format.replace("search_", "")}</span>
                  {s.is_default && (
                    <span className="text-xs bg-blue-500 text-white px-1.5 py-0.5 rounded">默认</span>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  {!s.is_default && (
                    <button onClick={() => handleSetDefaultSearch(s.id)} className="text-blue-400 hover:text-blue-300 text-xs">
                      设为默认
                    </button>
                  )}
                  <button onClick={() => handleDeleteSearch(s.id)} className="text-red-400 hover:text-red-300 text-xs">
                    删除
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        <div className="bg-white rounded-lg p-4 space-y-3">
          <div className="text-xs font-medium text-gray-500 mb-2">添加搜索引擎</div>
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
            <input className={inputCls} type="password" value={searchForm.api_key} onChange={(e) => setSearchForm({ ...searchForm, api_key: e.target.value })} />
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
              className="flex-1 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 text-sm py-1.5 rounded transition-colors"
            >
              {searchTesting ? "测试中..." : "测试连接"}
            </button>
            <button
              onClick={handleSaveSearch}
              disabled={!searchForm.name || !searchForm.api_format || !searchForm.api_key}
              className="flex-1 bg-blue-500 hover:bg-blue-600 disabled:opacity-50 text-white text-sm py-1.5 rounded transition-colors"
            >
              保存
            </button>
          </div>
        </div>
      </>)}
    </div>
  );
}
