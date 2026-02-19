import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  onInstalled: () => void;
  onClose: () => void;
}

export function InstallDialog({ onInstalled, onClose }: Props) {
  const [packPath, setPackPath] = useState("");
  const [username, setUsername] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function pickFile() {
    const f = await open({ filters: [{ name: "SkillPack", extensions: ["skillpack"] }] });
    if (f && typeof f === "string") setPackPath(f);
  }

  async function handleInstall() {
    if (!packPath || !username.trim()) {
      setError("请选择文件并填写用户名");
      return;
    }
    setLoading(true);
    setError("");
    try {
      await invoke("install_skill", { packPath, username });
      onInstalled();
      onClose();
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg p-6 w-96 space-y-4 border border-slate-600">
        <h2 className="font-semibold text-lg">安装 Skill</h2>
        <div>
          <button
            onClick={pickFile}
            className="w-full border border-dashed border-slate-500 rounded p-3 text-sm text-slate-400 hover:border-blue-500 hover:text-blue-400 transition-colors"
          >
            {packPath ? packPath.split(/[\/]/).pop() : "选择 .skillpack 文件"}
          </button>
        </div>
        <div>
          <label className="block text-xs text-slate-400 mb-1">用户名（创作者提供）</label>
          <input
            className="w-full bg-slate-700 border border-slate-600 rounded px-3 py-2 text-sm focus:outline-none focus:border-blue-500"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder=""
          />
        </div>
        {error && <div className="text-red-400 text-sm">{error}</div>}
        <div className="flex gap-2">
          <button
            onClick={onClose}
            className="flex-1 bg-slate-700 hover:bg-slate-600 py-2 rounded text-sm transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleInstall}
            disabled={loading}
            className="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 py-2 rounded text-sm transition-colors"
          >
            {loading ? "安装中..." : "安装"}
          </button>
        </div>
      </div>
    </div>
  );
}
