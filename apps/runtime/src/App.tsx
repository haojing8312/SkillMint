import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
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
        <InstallDialog onInstalled={loadSkills} onClose={() => setShowInstall(false)} />
      )}
    </div>
  );
}
