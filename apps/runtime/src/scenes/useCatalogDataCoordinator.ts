import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import type { Dispatch, SetStateAction } from "react";
import type { ModelConfig, SkillManifest } from "../types";
import { getDefaultSkillId } from "../app-shell-utils";
import {
  isTauriInvokeUnavailableError,
  isTauriRuntimeAvailable,
} from "../lib/tauriRuntime";

function warnCatalogLoadFailure(label: string, error: unknown) {
  if (isTauriInvokeUnavailableError(error)) {
    return;
  }
  console.warn(`${label}加载失败:`, error);
}

export function useCatalogDataCoordinator(options: {
  setSelectedSkillId: Dispatch<SetStateAction<string | null>>;
}) {
  const { setSelectedSkillId } = options;
  const [skills, setSkills] = useState<SkillManifest[]>([]);
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [searchConfigs, setSearchConfigs] = useState<ModelConfig[]>([]);
  const [hasHydratedModelConfigs, setHasHydratedModelConfigs] = useState(false);
  const [hasHydratedSearchConfigs, setHasHydratedSearchConfigs] =
    useState(false);

  const loadSkills = useCallback(async (): Promise<SkillManifest[]> => {
    if (!isTauriRuntimeAvailable()) {
      setSkills([]);
      setSelectedSkillId(null);
      return [];
    }

    try {
      const list = await invoke<SkillManifest[]>("list_skills");
      setSkills(list);
      setSelectedSkillId((prev) => {
        if (prev && list.some((item) => item.id === prev)) {
          return prev;
        }
        return getDefaultSkillId(list);
      });
      return list;
    } catch (error) {
      warnCatalogLoadFailure("技能目录", error);
      setSkills([]);
      setSelectedSkillId(null);
      return [];
    }
  }, [setSelectedSkillId]);

  const loadModels = useCallback(async () => {
    if (!isTauriRuntimeAvailable()) {
      setModels([]);
      setHasHydratedModelConfigs(true);
      return;
    }

    try {
      const list = await invoke<ModelConfig[]>("list_model_configs");
      setModels(Array.isArray(list) ? list : []);
    } catch (error) {
      warnCatalogLoadFailure("模型配置", error);
      setModels([]);
    } finally {
      setHasHydratedModelConfigs(true);
    }
  }, []);

  const loadSearchConfigs = useCallback(async () => {
    if (!isTauriRuntimeAvailable()) {
      setSearchConfigs([]);
      setHasHydratedSearchConfigs(true);
      return;
    }

    try {
      const list = await invoke<ModelConfig[]>("list_search_configs");
      setSearchConfigs(Array.isArray(list) ? list : []);
    } catch (error) {
      warnCatalogLoadFailure("搜索配置", error);
      setSearchConfigs([]);
    } finally {
      setHasHydratedSearchConfigs(true);
    }
  }, []);

  return {
    hasHydratedModelConfigs,
    hasHydratedSearchConfigs,
    loadModels,
    loadSearchConfigs,
    loadSkills,
    models,
    searchConfigs,
    skills,
  };
}
