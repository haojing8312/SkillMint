import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type InstallMode = "skillpack" | "local";

interface Props {
  onInstalled: () => void;
  onClose: () => void;
}

export function InstallDialog({ onInstalled, onClose }: Props) {
  const [mode, setMode] = useState<InstallMode>("skillpack");
  const [packPath, setPackPath] = useState("");
  const [username, setUsername] = useState("");
  const [localDir, setLocalDir] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  // 选择 .skillpack 文件
  async function pickFile() {
    const f = await open({ filters: [{ name: "SkillPack", extensions: ["skillpack"] }] });
    if (f && typeof f === "string") setPackPath(f);
  }

  // 选择本地 Skill 目录
  async function pickDir() {
    const d = await open({ directory: true });
    if (d && typeof d === "string") setLocalDir(d);
  }

  // 切换模式时清除错误
  function switchMode(m: InstallMode) {
    setMode(m);
    setError("");
  }

  async function handleInstall() {
    setError("");
    setLoading(true);

    try {
      if (mode === "skillpack") {
        if (!packPath || !username.trim()) {
          setError("请选择文件并填写用户名");
          setLoading(false);
          return;
        }
        await invoke("install_skill", { packPath, username });
      } else {
        if (!localDir) {
          setError("请选择包含 SKILL.md 的目录");
          setLoading(false);
          return;
        }
        await invoke("import_local_skill", { dirPath: localDir });
      }
      onInstalled();
      onClose();
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  const tabBase =
    "flex-1 py-1.5 text-sm rounded transition-colors text-center";
  const tabActive = "bg-blue-600 text-white";
  const tabInactive = "bg-slate-700 text-slate-400 hover:bg-slate-600";

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg p-6 w-96 space-y-4 border border-slate-600">
        <h2 className="font-semibold text-lg">安装 Skill</h2>

        {/* 模式切换 Tab */}
        <div className="flex gap-2">
          <button
            className={`${tabBase} ${mode === "skillpack" ? tabActive : tabInactive}`}
            onClick={() => switchMode("skillpack")}
          >
            加密 .skillpack
          </button>
          <button
            className={`${tabBase} ${mode === "local" ? tabActive : tabInactive}`}
            onClick={() => switchMode("local")}
          >
            本地目录
          </button>
        </div>

        {/* .skillpack 模式 */}
        {mode === "skillpack" && (
          <>
            <div>
              <button
                onClick={pickFile}
                className="w-full border border-dashed border-slate-500 rounded p-3 text-sm text-slate-400 hover:border-blue-500 hover:text-blue-400 transition-colors"
              >
                {packPath ? packPath.split(/[/\\]/).pop() : "选择 .skillpack 文件"}
              </button>
            </div>
            <div>
              <label className="block text-xs text-slate-400 mb-1">
                用户名（创作者提供）
              </label>
              <input
                className="w-full bg-slate-700 border border-slate-600 rounded px-3 py-2 text-sm focus:outline-none focus:border-blue-500"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder=""
              />
            </div>
          </>
        )}

        {/* 本地目录模式 */}
        {mode === "local" && (
          <>
            <div>
              <button
                onClick={pickDir}
                className="w-full border border-dashed border-slate-500 rounded p-3 text-sm text-slate-400 hover:border-blue-500 hover:text-blue-400 transition-colors"
              >
                {localDir ? localDir.split(/[/\\]/).pop() : "选择 Skill 目录"}
              </button>
              {localDir && (
                <div className="mt-1 text-xs text-slate-500 truncate" title={localDir}>
                  {localDir}
                </div>
              )}
            </div>
            <div className="text-xs text-slate-500">
              目录中需包含 <code className="text-slate-400">SKILL.md</code> 文件。
              本地 Skill 无需加密，可直接导入使用。
            </div>
          </>
        )}

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
