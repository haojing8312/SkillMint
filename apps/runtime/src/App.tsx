import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "./components/Sidebar";
import { ChatView } from "./components/ChatView";
import { InstallDialog } from "./components/InstallDialog";
import { SettingsView } from "./components/SettingsView";
import { SkillManifest, ModelConfig } from "./types";

export default function App() {
  const [skills, setSkills] = useState<SkillManifest[]>([]);
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);
  const [showInstall, setShowInstall] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    loadSkills();
    loadModels();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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

  const selectedSkill = skills.find((s) => s.id === selectedSkillId) ?? null;

  return (
    <div className="flex h-screen bg-slate-900 text-slate-100 overflow-hidden">
      <Sidebar
        skills={skills}
        selectedId={selectedSkillId}
        onSelect={setSelectedSkillId}
        onInstall={() => setShowInstall(true)}
        onSettings={() => setShowSettings(true)}
      />
      <div className="flex-1 overflow-hidden">
        {showSettings ? (
          <SettingsView onClose={async () => { await loadModels(); setShowSettings(false); }} />
        ) : selectedSkill && models.length > 0 ? (
          <ChatView skill={selectedSkill} models={models} />
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
        <InstallDialog
          onInstalled={loadSkills}
          onClose={() => setShowInstall(false)}
        />
      )}
    </div>
  );
}
