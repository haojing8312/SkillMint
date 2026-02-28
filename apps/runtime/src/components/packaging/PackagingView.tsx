import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { SkillDirInfo } from "../../types";
import { FileTree } from "./FileTree";
import { PackForm } from "./PackForm";

export function PackagingView() {
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
    <div className="flex flex-col h-full bg-gray-50 text-gray-800">
      <div className="flex items-center justify-between px-5 py-3 border-b border-gray-200 bg-white">
        <div className="flex items-center gap-2">
          <span className="text-blue-500 font-semibold text-base">技能打包</span>
        </div>
        <button
          onClick={handleSelectDir}
          className="bg-blue-500 hover:bg-blue-600 text-sm px-4 py-1.5 rounded-md font-medium text-white transition-colors"
        >
          选择技能目录
        </button>
      </div>

      {dirPath && (
        <div className="px-5 py-1.5 text-xs text-gray-500 border-b border-gray-200 bg-white/80 font-mono truncate">
          {dirPath}
        </div>
      )}

      {error && (
        <div className="mx-5 mt-3 text-red-600 text-sm bg-red-50 border border-red-200 rounded-md p-3">
          {error}
        </div>
      )}

      {!skillInfo && !error && (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center text-gray-400">
            <div className="text-4xl mb-3">[pkg]</div>
            <div className="text-sm">点击「选择技能目录」开始打包</div>
            <div className="text-xs mt-1 text-gray-400">目录中需要包含 SKILL.md</div>
          </div>
        </div>
      )}

      {skillInfo && dirPath && (
        <div className="flex flex-1 overflow-hidden">
          <div className="w-2/5 border-r border-gray-200 overflow-y-auto bg-white">
            <div className="px-4 py-2 text-xs font-semibold text-gray-500 border-b border-gray-200 bg-gray-50 uppercase tracking-wide">
              文件列表 {shortPath && <span className="font-normal normal-case text-gray-400">— {shortPath}</span>}
            </div>
            <FileTree files={skillInfo.files} />
          </div>
          <div className="w-3/5 overflow-y-auto bg-white">
            <div className="px-5 py-2 text-xs font-semibold text-gray-500 border-b border-gray-200 bg-gray-50 uppercase tracking-wide">
              打包配置
            </div>
            <div className="p-5">
              <PackForm dirPath={dirPath} frontMatter={skillInfo.front_matter} fileCount={skillInfo.files.length} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
