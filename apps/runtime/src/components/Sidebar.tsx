import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { SkillManifest, SessionInfo } from "../types";

interface Props {
  skills: SkillManifest[];
  selectedSkillId: string | null;
  onSelectSkill: (id: string) => void;
  sessions: SessionInfo[];
  selectedSessionId: string | null;
  onSelectSession: (id: string) => void;
  onNewSession: () => void;
  newSessionPermissionMode: "default" | "accept_edits" | "unrestricted";
  onChangeNewSessionPermissionMode: (mode: "default" | "accept_edits" | "unrestricted") => void;
  onDeleteSession: (id: string) => void;
  onInstall: () => void;
  onSettings: () => void;
  onSearchSessions: (query: string) => void;
  onExportSession: (sessionId: string) => void;
  onCollapse: () => void;
  collapsed: boolean;
}

export function Sidebar({
  skills,
  selectedSkillId,
  onSelectSkill,
  sessions,
  selectedSessionId,
  onSelectSession,
  onNewSession,
  newSessionPermissionMode,
  onChangeNewSessionPermissionMode,
  onDeleteSession,
  onInstall,
  onSettings,
  onSearchSessions,
  onExportSession,
  onCollapse,
  collapsed,
}: Props) {
  const [searchQuery, setSearchQuery] = useState("");

  function handleSearchChange(value: string) {
    setSearchQuery(value);
    onSearchSessions(value);
  }

  // 折叠模式：窄侧边栏，仅显示图标按钮
  if (collapsed) {
    return (
      <div className="w-12 bg-white flex flex-col h-full border-r border-gray-200 items-center py-3 gap-3 flex-shrink-0">
        <button
          onClick={onCollapse}
          className="w-8 h-8 flex items-center justify-center text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors"
          title="展开侧边栏"
          aria-label="展开侧边栏"
        >
          ▶
        </button>
        <button
          onClick={onInstall}
          className="w-8 h-8 flex items-center justify-center text-blue-500 hover:text-blue-400 hover:bg-blue-50 rounded transition-colors"
          title="安装 Skill"
          aria-label="安装 Skill"
        >
          +
        </button>
        <button
          onClick={onSettings}
          className="w-8 h-8 flex items-center justify-center text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors mt-auto"
          title="设置"
          aria-label="设置"
        >
          ⚙
        </button>
      </div>
    );
  }

  return (
    <div className="w-56 bg-white flex flex-col h-full border-r border-gray-200 flex-shrink-0">
      {/* 标题栏 + 折叠按钮 */}
      <div className="px-4 py-3 text-xs font-medium text-gray-500 border-b border-gray-200 flex items-center justify-between">
        <span>已安装 Skill</span>
        <button
          onClick={onCollapse}
          className="text-gray-500 hover:text-gray-600 text-sm transition-colors"
          title="折叠侧边栏"
        >
          ◀
        </button>
      </div>
      <div className="overflow-y-auto py-1" style={{ maxHeight: "30%" }}>
        {skills.length === 0 && (
          <div className="px-4 py-3 text-xs text-gray-400">暂无已安装 Skill</div>
        )}
        {[...skills].sort((a, b) => {
          if (a.id === "builtin-general") return -1;
          if (b.id === "builtin-general") return 1;
          return 0;
        }).map((s) => (
          <button
            key={s.id}
            onClick={() => onSelectSkill(s.id)}
            className={
              "w-full text-left px-4 py-2 text-sm transition-colors " +
              (selectedSkillId === s.id
                ? "bg-blue-50 text-blue-600"
                : "text-gray-700 hover:bg-gray-50")
            }
          >
            <div className="font-medium truncate flex items-center gap-1">
              {s.name}
              {s.id === "builtin-general" && (
                <span className="text-[10px] bg-blue-100 text-blue-600 px-1 py-0.5 rounded">
                  内置
                </span>
              )}
              {s.id.startsWith("local-") && (
                <span className="text-[10px] bg-green-100 text-green-600 px-1 py-0.5 rounded">
                  本地
                </span>
              )}
            </div>
            <div className="text-xs text-gray-400 truncate">{s.version}</div>
          </button>
        ))}
      </div>

      {/* 会话历史区域（仅在选中 Skill 后显示） */}
      {selectedSkillId && (
        <>
          <div className="px-4 py-2 text-xs font-medium text-gray-500 border-t border-b border-gray-200 flex items-center justify-between">
            <span>会话历史</span>
            <button
              onClick={onNewSession}
              className="text-blue-400 hover:text-blue-300 text-xs"
            >
              + 新建
            </button>
          </div>
          <div className="px-3 py-2 border-b border-gray-200">
            <label className="block text-[11px] text-gray-500 mb-1">新会话权限模式</label>
            <select
              value={newSessionPermissionMode}
              onChange={(e) =>
                onChangeNewSessionPermissionMode(
                  e.target.value as "default" | "accept_edits" | "unrestricted"
                )
              }
              className="w-full bg-gray-50 border border-gray-200 rounded px-2 py-1 text-xs text-gray-800 focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400"
            >
              <option value="accept_edits">accept_edits（推荐）</option>
              <option value="default">default（严格确认）</option>
              <option value="unrestricted">unrestricted（危险：跳过全部确认）</option>
            </select>
          </div>
          {/* 搜索框 */}
          <div className="px-3 py-2 border-b border-gray-200">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => handleSearchChange(e.target.value)}
              placeholder="搜索会话..."
              className="w-full bg-gray-50 border border-gray-200 rounded px-2 py-1 text-xs text-gray-800 placeholder-gray-400 focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400"
            />
          </div>
          <div className="flex-1 overflow-y-auto py-1">
            {sessions.length === 0 && (
              <div className="px-4 py-3 text-xs text-gray-400">
                {searchQuery ? "未找到匹配会话" : "暂无会话"}
              </div>
            )}
            <AnimatePresence>
              {sessions.map((s) => (
                <motion.div
                  key={s.id}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20, height: 0 }}
                  whileHover={{ scale: 1.01 }}
                  transition={{ duration: 0.2 }}
                  className={
                    "group flex items-center px-4 py-2 text-sm cursor-pointer transition-colors " +
                    (selectedSessionId === s.id
                      ? "bg-blue-50 text-blue-600"
                      : "text-gray-700 hover:bg-gray-50")
                  }
                  onClick={() => onSelectSession(s.id)}
                >
                  <div className="flex-1 min-w-0">
                    <div className="truncate text-xs">{s.title || "New Chat"}</div>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onExportSession(s.id);
                    }}
                    className="hidden group-hover:block text-gray-400 hover:text-gray-600 text-xs ml-1 flex-shrink-0"
                    title="导出会话"
                  >
                    ↓
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onDeleteSession(s.id);
                    }}
                    className="hidden group-hover:block text-red-400 hover:text-red-300 text-xs ml-1 flex-shrink-0"
                  >
                    ×
                  </button>
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        </>
      )}

      {/* 底部操作按钮 */}
      <div className="p-3 space-y-2 border-t border-gray-200">
        <button
          onClick={onInstall}
          className="w-full bg-blue-500 hover:bg-blue-600 active:scale-[0.97] text-white text-sm py-1.5 rounded-lg transition-all"
        >
          + 安装 Skill
        </button>
        <button
          onClick={onSettings}
          className="w-full bg-gray-100 hover:bg-gray-200 active:scale-[0.97] text-gray-700 text-sm py-1.5 rounded-lg transition-all"
        >
          设置
        </button>
      </div>
    </div>
  );
}
