import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ModelConfig } from "../types";

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

  useEffect(() => { loadModels(); }, []);

  async function loadModels() {
    const list = await invoke<ModelConfig[]>("list_model_configs");
    setModels(list);
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
    } catch {
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  }

  async function handleDelete(id: string) {
    await invoke("delete_model_config", { modelId: id });
    loadModels();
  }

  const inputCls = "w-full bg-slate-700 border border-slate-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500";
  const labelCls = "block text-xs text-slate-400 mb-1";

  return (
    <div className="flex flex-col h-full p-6 overflow-y-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold">模型配置</h2>
        <button onClick={onClose} className="text-slate-400 hover:text-white text-sm">
          返回
        </button>
      </div>

      {models.length > 0 && (
        <div className="mb-6 space-y-2">
          <div className="text-xs text-slate-400 mb-2">已配置模型</div>
          {models.map((m) => (
            <div key={m.id} className="flex items-center justify-between bg-slate-800 rounded px-3 py-2 text-sm">
              <div>
                <span className="font-medium">{m.name}</span>
                <span className="text-slate-400 ml-2">{m.model_name}</span>
              </div>
              <button onClick={() => handleDelete(m.id)} className="text-red-400 hover:text-red-300 text-xs">
                删除
              </button>
            </div>
          ))}
        </div>
      )}

      <div className="bg-slate-800 rounded-lg p-4 space-y-3">
        <div className="text-xs font-medium text-slate-400 mb-2">添加模型</div>
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
          <input className={inputCls} value={form.model_name} onChange={(e) => setForm({ ...form, model_name: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>API Key</label>
          <input className={inputCls} type="password" value={form.api_key} onChange={(e) => setForm({ ...form, api_key: e.target.value })} />
        </div>
        {error && <div className="text-red-400 text-xs">{error}</div>}
        {testResult !== null && (
          <div className={"text-xs " + (testResult ? "text-green-400" : "text-red-400")}>
            {testResult ? "连接成功" : "连接失败，请检查配置"}
          </div>
        )}
        <div className="flex gap-2 pt-1">
          <button
            onClick={handleTest}
            disabled={testing}
            className="flex-1 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 text-sm py-1.5 rounded transition-colors"
          >
            {testing ? "测试中..." : "测试连接"}
          </button>
          <button
            onClick={handleSave}
            className="flex-1 bg-blue-600 hover:bg-blue-700 text-sm py-1.5 rounded transition-colors"
          >
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
