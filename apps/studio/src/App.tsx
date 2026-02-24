import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FileTree } from "./components/FileTree";
import { PackForm } from "./components/PackForm";

interface FrontMatter {
  name?: string;
  description?: string;
  version?: string;
  model?: string;
}

interface SkillDirInfo {
  files: string[];
  front_matter: FrontMatter;
}

export default function App() {
  const [dirPath, setDirPath] = useState<string | null>(null);
  const [skillInfo, setSkillInfo] = useState<SkillDirInfo | null>(null);
  const [error, setError] = useState("");

  async function handleSelectDir() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || typeof selected !== "string") return;
    setError("");
    setSkillInfo(null);
    try {
      const info = await invoke<SkillDirInfo>("read_skill_dir", { dirPath: selected });
      setDirPath(selected);
      setSkillInfo(info);
    } catch (e: unknown) {
      setError(String(e));
    }
  }

  const shortPath = dirPath ? dirPath.split(/[\\/]/).slice(-2).join("/") : null;

  return (
    <div className="flex flex-col h-screen bg-slate-900 text-slate-100">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-slate-700/60 bg-slate-800/80">
        <div className="flex items-center gap-2">
          <span className="text-blue-400 font-bold text-base">SkillMint</span>
          <span className="text-slate-500 text-sm font-light">Studio</span>
        </div>
        <button
          onClick={handleSelectDir}
          className="bg-blue-600 hover:bg-blue-500 text-sm px-4 py-1.5 rounded-md font-medium transition-colors"
        >
          选择 Skill 目录
        </button>
      </div>

      {/* Path bar */}
      {dirPath && (
        <div className="px-5 py-1.5 text-xs text-slate-400 border-b border-slate-700/40 bg-slate-800/40 font-mono truncate">
          {dirPath}
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="mx-5 mt-3 text-red-400 text-sm bg-red-950/50 border border-red-800/50 rounded-md p-3">
          {error}
        </div>
      )}

      {/* Main content */}
      {!skillInfo && !error && (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center text-slate-500">
            <div className="text-4xl mb-3">[pkg]</div>
            <div className="text-sm">点击「选择 Skill 目录」开始打包</div>
            <div className="text-xs mt-1 text-slate-600">需要包含 SKILL.md 文件的目录</div>
          </div>
        </div>
      )}

      {skillInfo && (
        <div className="flex flex-1 overflow-hidden">
          {/* File tree panel */}
          <div className="w-2/5 border-r border-slate-700/60 overflow-y-auto">
            <div className="px-4 py-2 text-xs font-semibold text-slate-400 border-b border-slate-700/40 bg-slate-800/30 uppercase tracking-wide">
              文件树 {shortPath && <span className="font-normal normal-case text-slate-500">— {shortPath}</span>}
            </div>
            <FileTree files={skillInfo.files} />
          </div>

          {/* Pack form panel */}
          <div className="w-3/5 overflow-y-auto">
            <div className="px-5 py-2 text-xs font-semibold text-slate-400 border-b border-slate-700/40 bg-slate-800/30 uppercase tracking-wide">
              打包配置
            </div>
            <div className="p-5">
              <PackForm dirPath={dirPath!} frontMatter={skillInfo.front_matter} fileCount={skillInfo.files.length} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
