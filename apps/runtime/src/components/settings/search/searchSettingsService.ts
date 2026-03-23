import { invoke } from "@tauri-apps/api/core";
import type { ModelConfig } from "../../../types";
import type { SearchConfigFormState } from "../../../lib/search-config";

export interface SearchConfigDraft {
  id?: string;
  isDefault: boolean;
  form: SearchConfigFormState;
}

export async function listSearchConfigs() {
  return invoke<ModelConfig[]>("list_search_configs");
}

export async function getSearchConfigApiKey(configId: string) {
  return invoke<string>("get_model_api_key", { modelId: configId });
}

export async function saveSearchConfig(draft: SearchConfigDraft) {
  return invoke<string>("save_model_config", {
    config: {
      id: draft.id || "",
      name: draft.form.name,
      api_format: draft.form.api_format,
      base_url: draft.form.base_url,
      model_name: draft.form.model_name,
      is_default: draft.isDefault,
    },
    apiKey: draft.form.api_key,
  });
}

export async function testSearchConnection(form: SearchConfigFormState) {
  return invoke<boolean>("test_search_connection", {
    config: {
      id: "",
      name: form.name,
      api_format: form.api_format,
      base_url: form.base_url,
      model_name: form.model_name,
      is_default: false,
    },
    apiKey: form.api_key,
  });
}

export async function setDefaultSearchConfig(configId: string) {
  await invoke("set_default_search", { configId });
}

export async function deleteSearchConfig(configId: string) {
  await invoke("delete_model_config", { modelId: configId });
}
