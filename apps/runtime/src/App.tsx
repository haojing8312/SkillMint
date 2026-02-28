import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { motion, AnimatePresence } from "framer-motion";
import { Sidebar } from "./components/Sidebar";
import { ChatView } from "./components/ChatView";
import { InstallDialog } from "./components/InstallDialog";
import { SettingsView } from "./components/SettingsView";
import { PackagingView } from "./components/packaging/PackagingView";
import { SkillManifest, ModelConfig, SessionInfo } from "./types";

export default function App() {
  const [skills, setSkills] = useState<SkillManifest[]>([]);
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [showInstall, setShowInstall] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [activeMainView, setActiveMainView] = useState<"chat" | "packaging">("chat");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [newSessionPermissionMode, setNewSessionPermissionMode] = useState<"default" | "accept_edits" | "unrestricted">("accept_edits");
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

    // 弹出目录选择器
    const dir = await open({ directory: true, title: "选择工作目录" });
    if (!dir || typeof dir !== "string") return; // 用户取消

    try {
      const id = await invoke<string>("create_session", {
        skillId: selectedSkillId,
        modelId,
        workDir: dir,
        permissionMode: newSessionPermissionMode,
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
    await loadSkills();
    setSelectedSkillId(skillId);
    const modelId = models[0]?.id;
    if (modelId) {
      const dir = await open({ directory: true, title: "选择工作目录" });
      if (!dir || typeof dir !== "string") return;
      try {
        const sessionId = await invoke<string>("create_session", {
          skillId,
          modelId,
          workDir: dir,
          permissionMode: newSessionPermissionMode,
        });
        const sessions = await invoke<SessionInfo[]>("get_sessions", { skillId });
        setSessions(sessions);
        setSelectedSessionId(sessionId);
      } catch (e) {
        console.error("自动创建会话失败:", e);
      }
    }
  }

  const handleSessionRefresh = useCallback(() => {
    if (selectedSkillId) loadSessions(selectedSkillId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedSkillId]);

  const selectedSkill = skills.find((s) => s.id === selectedSkillId) ?? null;
  const selectedSession = sessions.find((s) => s.id === selectedSessionId);

  return (
    <div className="flex h-screen bg-gray-50 text-gray-800 overflow-hidden">
      <Sidebar
        activeMainView={activeMainView}
        onOpenChat={() => setActiveMainView("chat")}
        onOpenPackaging={() => {
          setShowSettings(false);
          setActiveMainView("packaging");
        }}
        skills={skills}
        selectedSkillId={selectedSkillId}
        onSelectSkill={setSelectedSkillId}
        sessions={sessions}
        selectedSessionId={selectedSessionId}
        onSelectSession={setSelectedSessionId}
        onNewSession={handleCreateSession}
        newSessionPermissionMode={newSessionPermissionMode}
        onChangeNewSessionPermissionMode={setNewSessionPermissionMode}
        onDeleteSession={handleDeleteSession}
        onInstall={() => setShowInstall(true)}
        onSettings={() => {
          setActiveMainView("chat");
          setShowSettings(true);
        }}
        onSearchSessions={handleSearchSessions}
        onExportSession={handleExportSession}
        onCollapse={() => setSidebarCollapsed((prev) => !prev)}
        collapsed={sidebarCollapsed}
      />
      <div className="flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          {showSettings ? (
            <motion.div
              key="settings"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="h-full"
            >
              <SettingsView
                onClose={async () => {
                  await loadModels();
                  setShowSettings(false);
                }}
              />
            </motion.div>
          ) : activeMainView === "packaging" ? (
            <motion.div
              key="packaging"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="h-full"
            >
              <PackagingView />
            </motion.div>
          ) : selectedSkill && models.length > 0 && selectedSessionId ? (
            <motion.div
              key="chat"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="h-full"
            >
              <ChatView
                skill={selectedSkill}
                models={models}
                sessionId={selectedSessionId}
                workDir={selectedSession?.work_dir}
                onSessionUpdate={handleSessionRefresh}
              />
            </motion.div>
          ) : selectedSkill && models.length > 0 ? (
            <motion.div
              key="new-session"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="flex items-center justify-center h-full text-gray-400 text-sm"
            >
              <button
                onClick={handleCreateSession}
                className="bg-blue-500 hover:bg-blue-600 active:scale-[0.97] px-4 py-2 rounded-lg text-white text-sm transition-all"
              >
                新建会话
              </button>
            </motion.div>
          ) : selectedSkill && models.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-400 text-sm">
              请先在设置中配置模型和 API Key
            </div>
          ) : (
            <div className="flex items-center justify-center h-full text-gray-400 text-sm">
              从左侧选择一个 Skill 开始对话
            </div>
          )}
        </AnimatePresence>
      </div>
      {showInstall && (
        <InstallDialog onInstalled={handleInstalled} onClose={() => setShowInstall(false)} />
      )}
    </div>
  );
}
