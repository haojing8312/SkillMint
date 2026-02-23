import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Sidebar } from "./components/Sidebar";
import { ChatView } from "./components/ChatView";
import { InstallDialog } from "./components/InstallDialog";
import { SettingsView } from "./components/SettingsView";
import { SkillManifest, ModelConfig, SessionInfo } from "./types";

export default function App() {
  const [skills, setSkills] = useState<SkillManifest[]>([]);
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [showInstall, setShowInstall] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    loadSkills();
    loadModels();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (selectedSkillId) {
      loadSessions(selectedSkillId);
    } else {
      setSessions([]);
      setSelectedSessionId(null);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedSkillId]);

  async function loadSkills() {
    const list = await invoke<SkillManifest[]>("list_skills");
    setSkills(list);
    if (list.length > 0 && !selectedSkillId) {
      setSelectedSkillId(list[0].id);
    }
  }

  async function loadModels() {
    const list = await invoke<ModelConfig[]>("list_model_configs");
    setModels(list);
  }

  async function loadSessions(skillId: string) {
    try {
      const list = await invoke<SessionInfo[]>("get_sessions", { skillId });
      setSessions(list);
    } catch (e) {
      console.error("加载会话列表失败:", e);
      setSessions([]);
    }
  }

  async function handleCreateSession() {
    const modelId = models[0]?.id;
    if (!selectedSkillId || !modelId) return;
    try {
      const id = await invoke<string>("create_session", {
        skillId: selectedSkillId,
        modelId,
      });
      setSelectedSessionId(id);
      if (selectedSkillId) await loadSessions(selectedSkillId);
    } catch (e) {
      console.error("创建会话失败:", e);
    }
  }

  async function handleDeleteSession(sessionId: string) {
    try {
      await invoke("delete_session", { sessionId });
      if (selectedSessionId === sessionId) setSelectedSessionId(null);
      if (selectedSkillId) await loadSessions(selectedSkillId);
    } catch (e) {
      console.error("删除会话失败:", e);
    }
  }

  // 搜索会话（300ms debounce）
  function handleSearchSessions(query: string) {
    if (searchTimerRef.current) {
      clearTimeout(searchTimerRef.current);
    }
    if (!selectedSkillId) return;

    if (!query.trim()) {
      // 搜索词为空时恢复完整会话列表
      searchTimerRef.current = setTimeout(() => {
        loadSessions(selectedSkillId!);
      }, 100);
      return;
    }

    searchTimerRef.current = setTimeout(async () => {
      try {
        const results = await invoke<SessionInfo[]>("search_sessions", {
          skillId: selectedSkillId,
          query: query.trim(),
        });
        setSessions(results);
      } catch (e) {
        console.error("搜索会话失败:", e);
      }
    }, 300);
  }

  // 导出会话为 Markdown 文件
  async function handleExportSession(sessionId: string) {
    try {
      const md = await invoke<string>("export_session", { sessionId });
      const filePath = await save({
        defaultPath: "session-export.md",
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });
      if (filePath) {
        await invoke("write_export_file", { path: filePath, content: md });
      }
    } catch (e) {
      console.error("导出会话失败:", e);
    }
  }

  // 安装 Skill 后自动切换并创建新会话
  async function handleInstalled(skillId: string) {
    try {
      await loadSkills();
      setSelectedSkillId(skillId);

      // 自动创建新会话
      const modelId = models[0]?.id;
      if (modelId) {
        const sessionId = await invoke<string>("create_session", {
          skillId,
          modelId,
        });
        setSelectedSessionId(sessionId);
        await loadSessions(skillId);
      }
    } catch (e) {
      console.error("安装后自动创建会话失败:", e);
    }
  }

  const handleSessionRefresh = useCallback(() => {
    if (selectedSkillId) loadSessions(selectedSkillId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedSkillId]);

  const selectedSkill = skills.find((s) => s.id === selectedSkillId) ?? null;

  return (
    <div className="flex h-screen bg-slate-900 text-slate-100 overflow-hidden">
      <Sidebar
        skills={skills}
        selectedSkillId={selectedSkillId}
        onSelectSkill={setSelectedSkillId}
        sessions={sessions}
        selectedSessionId={selectedSessionId}
        onSelectSession={setSelectedSessionId}
        onNewSession={handleCreateSession}
        onDeleteSession={handleDeleteSession}
        onInstall={() => setShowInstall(true)}
        onSettings={() => setShowSettings(true)}
        onSearchSessions={handleSearchSessions}
        onExportSession={handleExportSession}
        onCollapse={() => setSidebarCollapsed((prev) => !prev)}
        collapsed={sidebarCollapsed}
      />
      <div className="flex-1 overflow-hidden">
        {showSettings ? (
          <SettingsView
            onClose={async () => {
              await loadModels();
              setShowSettings(false);
            }}
          />
        ) : selectedSkill && models.length > 0 && selectedSessionId ? (
          <ChatView
            skill={selectedSkill}
            models={models}
            sessionId={selectedSessionId}
            onSessionUpdate={handleSessionRefresh}
          />
        ) : selectedSkill && models.length > 0 ? (
          <div className="flex items-center justify-center h-full text-slate-400 text-sm">
            <button
              onClick={handleCreateSession}
              className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded text-white text-sm"
            >
              新建会话
            </button>
          </div>
        ) : selectedSkill && models.length === 0 ? (
          <div className="flex items-center justify-center h-full text-slate-400 text-sm">
            请先在设置中配置模型和 API Key
          </div>
        ) : (
          <div className="flex items-center justify-center h-full text-slate-400 text-sm">
            从左侧选择一个 Skill 开始对话
          </div>
        )}
      </div>
      {showInstall && (
        <InstallDialog onInstalled={handleInstalled} onClose={() => setShowInstall(false)} />
      )}
    </div>
  );
}
