import { useEffect, useState } from "react";
import { SearchConfigForm } from "../../SearchConfigForm";
import { EMPTY_SEARCH_CONFIG_FORM, applySearchPresetToForm, validateSearchConfigForm } from "../../../lib/search-config";
import type { ModelConfig } from "../../../types";
import {
  deleteSearchConfig,
  getSearchConfigApiKey,
  listSearchConfigs,
  saveSearchConfig,
  setDefaultSearchConfig,
  testSearchConnection,
} from "./searchSettingsService";

export function SearchSettingsSection() {
  const [searchConfigs, setSearchConfigs] = useState<ModelConfig[]>([]);
  const [searchForm, setSearchForm] = useState(EMPTY_SEARCH_CONFIG_FORM);
  const [searchError, setSearchError] = useState("");
  const [searchTesting, setSearchTesting] = useState(false);
  const [searchTestResult, setSearchTestResult] = useState<boolean | null>(null);
  const [editingSearchId, setEditingSearchId] = useState<string | null>(null);
  const [showSearchApiKey, setShowSearchApiKey] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadSearchConfigs() {
      try {
        const list = await listSearchConfigs();
        if (!cancelled) {
          setSearchConfigs((current) => (current.length === 0 && list.length === 0 ? current : list));
        }
      } catch (cause) {
        console.error("加载搜索引擎配置失败:", cause);
      }
    }

    void loadSearchConfigs();
    return () => {
      cancelled = true;
    };
  }, []);

  async function refreshSearchConfigs() {
    try {
      const list = await listSearchConfigs();
      setSearchConfigs((current) => (current.length === 0 && list.length === 0 ? current : list));
    } catch (cause) {
      console.error("加载搜索引擎配置失败:", cause);
    }
  }

  async function handleEditSearch(s: ModelConfig) {
    try {
      const apiKey = await getSearchConfigApiKey(s.id);
      setSearchForm({
        name: s.name,
        api_format: s.api_format,
        base_url: s.base_url,
        model_name: s.model_name,
        api_key: apiKey,
      });
      setEditingSearchId(s.id);
      setShowSearchApiKey(false);
      setSearchError("");
      setSearchTestResult(null);
    } catch (cause) {
      setSearchError("加载配置失败: " + String(cause));
    }
  }

  function handleCancelEdit() {
    setEditingSearchId(null);
    setShowSearchApiKey(false);
    setSearchForm(EMPTY_SEARCH_CONFIG_FORM);
    setSearchError("");
    setSearchTestResult(null);
  }

  function applySearchPreset(value: string) {
    setSearchForm((current) => applySearchPresetToForm(value, current));
  }

  async function handleSaveSearch() {
    const validationError = validateSearchConfigForm(searchForm);
    if (validationError) {
      setSearchError(validationError);
      setSearchTestResult(null);
      return;
    }

    setSearchError("");
    try {
      await saveSearchConfig({
        id: editingSearchId || undefined,
        isDefault: editingSearchId
          ? searchConfigs.find((item) => item.id === editingSearchId)?.is_default ?? false
          : searchConfigs.length === 0,
        form: searchForm,
      });
      handleCancelEdit();
      await refreshSearchConfigs();
    } catch (cause) {
      setSearchError(String(cause));
    }
  }

  async function handleTestSearch() {
    const validationError = validateSearchConfigForm(searchForm);
    if (validationError) {
      setSearchError(validationError);
      setSearchTestResult(null);
      return;
    }

    setSearchError("");
    setSearchTesting(true);
    setSearchTestResult(null);
    try {
      const ok = await testSearchConnection(searchForm);
      setSearchTestResult(ok);
    } catch (cause) {
      setSearchError(String(cause));
      setSearchTestResult(false);
    } finally {
      setSearchTesting(false);
    }
  }

  async function handleSetDefaultSearch(id: string) {
    await setDefaultSearchConfig(id);
    await refreshSearchConfigs();
  }

  async function handleDeleteSearch(id: string) {
    await deleteSearchConfig(id);
    if (editingSearchId === id) {
      handleCancelEdit();
    }
    await refreshSearchConfigs();
  }

  return (
    <>
      {searchConfigs.length > 0 && (
        <div className="mb-6 space-y-2">
          <div className="text-xs text-gray-500 mb-2">已配置搜索引擎</div>
          {searchConfigs.map((s) => (
            <div
              key={s.id}
              className={
                "flex items-center justify-between bg-white rounded-lg px-4 py-2.5 text-sm border transition-colors " +
                (editingSearchId === s.id ? "border-blue-400 ring-1 ring-blue-400" : "border-transparent hover:border-gray-200")
              }
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-gray-800">{s.name}</span>
                  {s.is_default && <span className="text-[10px] bg-blue-500 text-white px-1.5 py-0.5 rounded">默认</span>}
                </div>
                <div className="text-xs text-gray-400 mt-0.5 truncate">
                  {s.api_format.replace("search_", "")} · {s.base_url}
                </div>
              </div>
              <div className="flex items-center gap-2 flex-shrink-0 ml-3">
                {!s.is_default && (
                  <button onClick={() => void handleSetDefaultSearch(s.id)} className="text-blue-400 hover:text-blue-500 text-xs">
                    设为默认
                  </button>
                )}
                <button onClick={() => void handleEditSearch(s)} className="text-blue-500 hover:text-blue-600 text-xs">
                  编辑
                </button>
                <button onClick={() => void handleDeleteSearch(s.id)} className="text-red-400 hover:text-red-500 text-xs">
                  删除
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="bg-white rounded-lg p-4 space-y-3">
        <div className="flex items-center justify-between mb-2">
          <div className="text-xs font-medium text-gray-500">{editingSearchId ? "编辑搜索引擎" : "添加搜索引擎"}</div>
          {editingSearchId && (
            <button onClick={handleCancelEdit} className="text-xs text-gray-400 hover:text-gray-600">
              取消编辑
            </button>
          )}
        </div>
        <SearchConfigForm
          form={searchForm}
          onFormChange={setSearchForm}
          onApplyPreset={applySearchPreset}
          showApiKey={showSearchApiKey}
          onToggleApiKey={() => setShowSearchApiKey((value) => !value)}
          error={searchError}
          testResult={searchTestResult}
          testing={searchTesting}
          saving={false}
          onTest={() => void handleTestSearch()}
          onSave={() => void handleSaveSearch()}
          labelClassName="sm-field-label"
          inputClassName="sm-input w-full text-sm py-1.5"
          panelClassName="space-y-3"
          actionClassName="flex gap-2 pt-1"
          saveLabel={editingSearchId ? "保存修改" : "保存"}
        />
      </div>
    </>
  );
}
