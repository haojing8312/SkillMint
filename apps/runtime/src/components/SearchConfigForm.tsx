import { Eye, EyeOff } from "lucide-react";
import type { SearchConfigFormState } from "../lib/search-config";
import { SEARCH_PRESETS } from "../lib/search-config";

interface SearchConfigFormProps {
  form: SearchConfigFormState;
  onFormChange: (next: SearchConfigFormState) => void;
  onApplyPreset: (value: string) => void;
  showApiKey: boolean;
  onToggleApiKey: () => void;
  error: string;
  testResult: boolean | null;
  testing: boolean;
  saving: boolean;
  onTest: () => void;
  onSave: () => void;
  labelClassName?: string;
  inputClassName?: string;
  panelClassName?: string;
  actionClassName?: string;
  saveLabel?: string;
  testLabel?: string;
  disabled?: boolean;
  onSecondaryAction?: () => void;
  secondaryActionLabel?: string;
}

export function SearchConfigForm({
  form,
  onFormChange,
  onApplyPreset,
  showApiKey,
  onToggleApiKey,
  error,
  testResult,
  testing,
  saving,
  onTest,
  onSave,
  labelClassName = "sm-field-label",
  inputClassName = "sm-input w-full text-sm py-1.5",
  panelClassName = "",
  actionClassName = "flex gap-2 pt-1",
  saveLabel = "保存",
  testLabel = "测试连接",
  disabled = false,
  onSecondaryAction,
  secondaryActionLabel,
}: SearchConfigFormProps) {
  const isBusy = disabled || testing || saving;

  return (
    <div className={panelClassName}>
      <div>
        <label className={labelClassName}>快速选择搜索引擎</label>
        <select
          aria-label="快速选择搜索引擎"
          className={inputClassName}
          value={SEARCH_PRESETS.some((item) => item.api_format === form.api_format) ? SEARCH_PRESETS.find((item) => item.api_format === form.api_format)?.value ?? "" : ""}
          onChange={(event) => onApplyPreset(event.target.value)}
          disabled={disabled}
        >
          {SEARCH_PRESETS.map((preset) => (
            <option key={preset.value} value={preset.value}>
              {preset.label}
            </option>
          ))}
        </select>
      </div>
      <div>
        <label className={labelClassName}>名称</label>
        <input
          aria-label="名称"
          className={inputClassName}
          value={form.name}
          onChange={(event) => onFormChange({ ...form, name: event.target.value })}
          disabled={disabled}
        />
      </div>
      <div>
        <label className={labelClassName}>API Key</label>
        <div className="relative">
          <input
            aria-label="API Key"
            className={`${inputClassName} pr-10`}
            type={showApiKey ? "text" : "password"}
            value={form.api_key}
            onChange={(event) => onFormChange({ ...form, api_key: event.target.value })}
            disabled={disabled}
          />
          <button
            type="button"
            onClick={onToggleApiKey}
            className="sm-btn sm-btn-ghost absolute right-2 top-1/2 h-8 w-8 -translate-y-1/2 rounded-lg p-0 text-gray-400 hover:text-gray-600"
            title={showApiKey ? "隐藏" : "显示"}
            aria-label={showApiKey ? "隐藏 API Key" : "显示 API Key"}
            disabled={disabled}
          >
            {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
      </div>
      <div>
        <label className={labelClassName}>Base URL（可选自定义）</label>
        <input
          aria-label="Base URL（可选自定义）"
          className={inputClassName}
          value={form.base_url}
          onChange={(event) => onFormChange({ ...form, base_url: event.target.value })}
          disabled={disabled}
        />
      </div>
      {form.api_format === "search_serpapi" && (
        <div>
          <label className={labelClassName}>搜索引擎 (google/baidu/bing)</label>
          <input
            aria-label="搜索引擎 (google/baidu/bing)"
            className={inputClassName}
            value={form.model_name}
            onChange={(event) => onFormChange({ ...form, model_name: event.target.value })}
            disabled={disabled}
          />
        </div>
      )}
      {error ? <div className="bg-red-50 text-red-600 text-xs px-2 py-1 rounded">{error}</div> : null}
      {testResult !== null ? (
        <div className={`text-xs px-2 py-1 rounded ${testResult ? "bg-green-50 text-green-600" : "bg-red-50 text-red-600"}`}>
          {testResult ? "连接成功" : "连接失败，请检查配置"}
        </div>
      ) : null}
      <div className={actionClassName}>
        {onSecondaryAction && secondaryActionLabel ? (
          <button
            type="button"
            onClick={onSecondaryAction}
            disabled={isBusy}
            className="flex-1 bg-white hover:bg-gray-50 disabled:opacity-50 text-sm py-1.5 rounded-lg border border-gray-200 transition-all active:scale-[0.97]"
          >
            {secondaryActionLabel}
          </button>
        ) : null}
        <button
          type="button"
          onClick={onTest}
          disabled={isBusy || !form.api_format}
          className="flex-1 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
        >
          {testing ? "测试中..." : testLabel}
        </button>
        <button
          type="button"
          onClick={onSave}
          disabled={isBusy || !form.name || !form.api_format || !form.api_key}
          className="flex-1 bg-blue-500 hover:bg-blue-600 disabled:opacity-50 text-white text-sm py-1.5 rounded-lg transition-all active:scale-[0.97]"
        >
          {saving ? "保存中..." : saveLabel}
        </button>
      </div>
    </div>
  );
}
