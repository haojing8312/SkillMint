import { invoke } from "@tauri-apps/api/core";
import {
  DEFAULT_MODEL_PROVIDER_ID,
  buildModelFormFromCatalogItem,
  getModelProviderCatalogItem,
  resolveCatalogItemForProviderIdentity,
} from "../../../model-provider-catalog";
import type {
  ModelConfig,
  ModelConnectionTestResult,
  ProviderConfig,
} from "../../../types";

export interface ModelFormState {
  name: string;
  api_format: "openai" | "anthropic";
  base_url: string;
  model_name: string;
  api_key: string;
}

export interface SaveModelDraft {
  id?: string;
  isDefault: boolean;
  form: ModelFormState;
}

export function getDefaultModelForm(providerId = DEFAULT_MODEL_PROVIDER_ID): ModelFormState {
  const provider = getModelProviderCatalogItem(providerId);
  return {
    ...buildModelFormFromCatalogItem(provider),
    api_key: "",
  };
}

export function validateModelForm(form: ModelFormState): string | null {
  if (!form.name.trim()) {
    return "请输入名称";
  }
  if (!form.base_url.trim()) {
    return "请输入 Base URL";
  }
  if (!form.model_name.trim()) {
    return "请输入模型名称";
  }
  if (!form.api_key.trim()) {
    return "请输入 API Key";
  }
  return null;
}

export function inferConnectionKey(baseUrl: string, apiFormat: string): string {
  const normalized = (baseUrl || "").toLowerCase();
  if (normalized.includes("deepseek")) return "deepseek";
  if (normalized.includes("dashscope")) return "qwen";
  if (normalized.includes("moonshot") || normalized.includes("kimi")) return "moonshot";
  if (normalized.includes("bigmodel") || normalized.includes("open.bigmodel")) return "zhipu";
  if (normalized.includes("anthropic")) return "anthropic";
  if (normalized.includes("minimax")) return "minimax";
  if (normalized.includes("lingyiwanwu")) return "yi";
  if (normalized.includes("openai")) return "openai";
  if (apiFormat === "anthropic") return "anthropic";
  return "openai";
}

export function resolveModelProviderForEdit(
  model: ModelConfig,
  providers: ProviderConfig[],
) {
  const apiFormat = model.api_format === "anthropic" ? "anthropic" : "openai";
  const providerConfig = providers.find((item) => item.id === model.id);
  return resolveCatalogItemForProviderIdentity({
    providerKey: providerConfig?.provider_key,
    apiFormat,
    baseUrl: model.base_url,
  });
}

export async function syncConnectionToRouting(
  model: ModelConfig,
  apiKey: string,
  preferredProviderKey?: string,
) {
  await invoke("save_provider_config", {
    config: {
      id: model.id,
      provider_key: preferredProviderKey || inferConnectionKey(model.base_url, model.api_format),
      display_name: model.name || model.model_name || model.id,
      protocol_type: model.api_format === "anthropic" ? "anthropic" : "openai",
      base_url: model.base_url,
      auth_type: "api_key",
      api_key_encrypted: apiKey,
      org_id: "",
      extra_json: "{}",
      enabled: true,
    },
  });
}

export async function listModelConfigs() {
  return invoke<ModelConfig[]>("list_model_configs");
}

export async function listProviderConfigs() {
  return invoke<ProviderConfig[]>("list_provider_configs");
}

export async function getModelApiKey(modelId: string) {
  return invoke<string>("get_model_api_key", { modelId });
}

export async function saveModelConfig(draft: SaveModelDraft) {
  const savedModelId = await invoke<string>("save_model_config", {
    config: {
      id: draft.id || "",
      name: draft.form.name.trim(),
      api_format: draft.form.api_format,
      base_url: draft.form.base_url.trim(),
      model_name: draft.form.model_name.trim(),
      is_default: draft.isDefault,
    },
    apiKey: draft.form.api_key.trim(),
  });
  return savedModelId;
}

export async function testModelConnection(form: ModelFormState) {
  return invoke<ModelConnectionTestResult>("test_connection_cmd", {
    config: {
      id: "",
      name: form.name.trim(),
      api_format: form.api_format,
      base_url: form.base_url.trim(),
      model_name: form.model_name.trim(),
      is_default: false,
    },
    apiKey: form.api_key.trim(),
  });
}

export async function deleteModelConfig(id: string) {
  await invoke("delete_model_config", { modelId: id });
  await invoke("delete_provider_config", { providerId: id }).catch(() => null);
}

export async function setDefaultModel(id: string) {
  await invoke("set_default_model", { modelId: id });
}

export async function syncModelConnections(modelList: ModelConfig[]) {
  let existingProviders: ProviderConfig[] = [];
  try {
    existingProviders = await listProviderConfigs();
  } catch (error) {
    console.warn("读取已保存 Provider 配置失败:", error);
  }

  await Promise.all(
    modelList.map(async (model) => {
      try {
        const apiKey = await getModelApiKey(model.id);
        const existingProviderKey = existingProviders.find((provider) => provider.id === model.id)?.provider_key;
        await syncConnectionToRouting(model, apiKey, existingProviderKey);
      } catch (error) {
        console.warn("同步连接配置失败:", model.id, error);
      }
    }),
  );
}
