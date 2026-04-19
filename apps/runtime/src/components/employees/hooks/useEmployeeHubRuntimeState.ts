import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  extractFeishuRegistryEntry,
  extractFeishuRuntimeStatusFromEntry,
  loadImChannelRegistry,
} from "../../settings/channels/channelRegistryService";
import {
  OpenClawPluginFeishuRuntimeStatus,
  RuntimePreferences,
} from "../../../types";

export interface UseEmployeeHubRuntimeStateArgs {
  setMessage: (message: string) => void;
}

export function useEmployeeHubRuntimeState({ setMessage }: UseEmployeeHubRuntimeStateArgs) {
  const [globalDefaultWorkDir, setGlobalDefaultWorkDir] = useState("");
  const [savingGlobalWorkDir, setSavingGlobalWorkDir] = useState(false);
  const [officialFeishuRuntimeStatus, setOfficialFeishuRuntimeStatus] =
    useState<OpenClawPluginFeishuRuntimeStatus | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const prefs = await invoke<RuntimePreferences>("get_runtime_preferences");
        setGlobalDefaultWorkDir(prefs.default_work_dir || "");
      } catch {
        // ignore
      }
    })();
  }, []);

  useEffect(() => {
    let disposed = false;
    const loadStatuses = async () => {
      try {
        const entries = await loadImChannelRegistry().catch(() => []);
        if (!disposed) {
          setOfficialFeishuRuntimeStatus(
            extractFeishuRuntimeStatusFromEntry(extractFeishuRegistryEntry(entries)),
          );
        }
      } catch {
        if (!disposed) {
          setOfficialFeishuRuntimeStatus(null);
        }
      }
    };
    void loadStatuses();
    const timer = setInterval(() => {
      void loadStatuses();
    }, 5000);
    return () => {
      disposed = true;
      clearInterval(timer);
    };
  }, []);

  async function saveGlobalDefaultWorkDir() {
    if (!globalDefaultWorkDir.trim()) {
      setMessage("默认工作目录不能为空");
      return;
    }
    setSavingGlobalWorkDir(true);
    setMessage("");
    try {
      await invoke("set_runtime_preferences", { input: { default_work_dir: globalDefaultWorkDir.trim() } });
      const resolved = await invoke<string>("resolve_default_work_dir");
      setGlobalDefaultWorkDir(resolved);
      setMessage("全局默认工作目录已保存");
    } catch (e) {
      setMessage(String(e));
    } finally {
      setSavingGlobalWorkDir(false);
    }
  }

  return {
    globalDefaultWorkDir,
    setGlobalDefaultWorkDir,
    savingGlobalWorkDir,
    officialFeishuRuntimeStatus,
    saveGlobalDefaultWorkDir,
  };
}
