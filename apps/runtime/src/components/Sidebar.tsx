import { SkillManifest } from "../types";

interface Props {
  skills: SkillManifest[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onInstall: () => void;
  onSettings: () => void;
}

export function Sidebar({ skills, selectedId, onSelect, onInstall, onSettings }: Props) {
  return (
    <div className="w-56 bg-slate-800 flex flex-col h-full border-r border-slate-700">
      <div className="px-4 py-3 text-xs font-medium text-slate-400 border-b border-slate-700">
        已安装 Skill
      </div>
      <div className="flex-1 overflow-y-auto py-2">
        {skills.length === 0 && (
          <div className="px-4 py-3 text-xs text-slate-500">暂无已安装 Skill</div>
        )}
        {skills.map((s) => (
          <button
            key={s.id}
            onClick={() => onSelect(s.id)}
            className={
              "w-full text-left px-4 py-2.5 text-sm transition-colors " +
              (selectedId === s.id
                ? "bg-blue-600/30 text-blue-300"
                : "text-slate-300 hover:bg-slate-700")
            }
          >
            <div className="font-medium truncate">{s.name}</div>
            <div className="text-xs text-slate-500 truncate">{s.version}</div>
          </button>
        ))}
      </div>
      <div className="p-3 space-y-2 border-t border-slate-700">
        <button
          onClick={onInstall}
          className="w-full bg-blue-600 hover:bg-blue-700 text-sm py-1.5 rounded transition-colors"
        >
          + 安装 Skill
        </button>
        <button
          onClick={onSettings}
          className="w-full bg-slate-700 hover:bg-slate-600 text-sm py-1.5 rounded transition-colors"
        >
          设置
        </button>
      </div>
    </div>
  );
}
