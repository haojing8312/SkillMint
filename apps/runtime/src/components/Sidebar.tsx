import { SkillManifest, SessionInfo } from "../types";

interface Props {
  skills: SkillManifest[];
  selectedSkillId: string | null;
  onSelectSkill: (id: string) => void;
  sessions: SessionInfo[];
  selectedSessionId: string | null;
  onSelectSession: (id: string) => void;
  onNewSession: () => void;
  onDeleteSession: (id: string) => void;
  onInstall: () => void;
  onSettings: () => void;
}

export function Sidebar({
  skills,
  selectedSkillId,
  onSelectSkill,
  sessions,
  selectedSessionId,
  onSelectSession,
  onNewSession,
  onDeleteSession,
  onInstall,
  onSettings,
}: Props) {
  return (
    <div className="w-56 bg-slate-800 flex flex-col h-full border-r border-slate-700">
      {/* Skill 列表区域 */}
      <div className="px-4 py-3 text-xs font-medium text-slate-400 border-b border-slate-700">
        已安装 Skill
      </div>
      <div className="overflow-y-auto py-1" style={{ maxHeight: "30%" }}>
        {skills.length === 0 && (
          <div className="px-4 py-3 text-xs text-slate-500">暂无已安装 Skill</div>
        )}
        {skills.map((s) => (
          <button
            key={s.id}
            onClick={() => onSelectSkill(s.id)}
            className={
              "w-full text-left px-4 py-2 text-sm transition-colors " +
              (selectedSkillId === s.id
                ? "bg-blue-600/30 text-blue-300"
                : "text-slate-300 hover:bg-slate-700")
            }
          >
            <div className="font-medium truncate flex items-center gap-1">
              {s.name}
              {s.id.startsWith("local-") && (
                <span className="text-[10px] bg-green-800/60 text-green-300 px-1 py-0.5 rounded">
                  本地
                </span>
              )}
            </div>
            <div className="text-xs text-slate-500 truncate">{s.version}</div>
          </button>
        ))}
      </div>

      {/* 会话历史区域（仅在选中 Skill 后显示） */}
      {selectedSkillId && (
        <>
          <div className="px-4 py-2 text-xs font-medium text-slate-400 border-t border-b border-slate-700 flex items-center justify-between">
            <span>会话历史</span>
            <button
              onClick={onNewSession}
              className="text-blue-400 hover:text-blue-300 text-xs"
            >
              + 新建
            </button>
          </div>
          <div className="flex-1 overflow-y-auto py-1">
            {sessions.length === 0 && (
              <div className="px-4 py-3 text-xs text-slate-500">暂无会话</div>
            )}
            {sessions.map((s) => (
              <div
                key={s.id}
                className={
                  "group flex items-center px-4 py-2 text-sm cursor-pointer transition-colors " +
                  (selectedSessionId === s.id
                    ? "bg-blue-600/20 text-blue-300"
                    : "text-slate-300 hover:bg-slate-700")
                }
                onClick={() => onSelectSession(s.id)}
              >
                <div className="flex-1 min-w-0">
                  <div className="truncate text-xs">{s.title || "New Chat"}</div>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteSession(s.id);
                  }}
                  className="hidden group-hover:block text-red-400 hover:text-red-300 text-xs ml-1 flex-shrink-0"
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        </>
      )}

      {/* 底部操作按钮 */}
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
