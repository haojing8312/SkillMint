import type { OpenClawPluginFeishuAdvancedSettings } from "../../../types";
import type { FeishuAdvancedFieldConfig } from "./FeishuAdvancedSection.types";

interface FeishuAdvancedFieldEditorProps {
  field: FeishuAdvancedFieldConfig;
  feishuAdvancedSettings: OpenClawPluginFeishuAdvancedSettings;
  onUpdateFeishuAdvancedSettings: (patch: Partial<OpenClawPluginFeishuAdvancedSettings>) => void;
}

export function FeishuAdvancedFieldEditor({
  field,
  feishuAdvancedSettings,
  onUpdateFeishuAdvancedSettings,
}: FeishuAdvancedFieldEditorProps) {
  const value = feishuAdvancedSettings[field.key];
  const updateValue = (nextValue: string) => onUpdateFeishuAdvancedSettings({ [field.key]: nextValue });

  return (
    <label className="space-y-1.5">
      <div className="flex items-center justify-between gap-3">
        <div className="text-[11px] font-medium text-gray-700">{field.label}</div>
        <div className="text-[10px] text-gray-400">{field.kind === "textarea" ? "JSON / 模板" : "文本值"}</div>
      </div>
      <div className="text-[11px] leading-5 text-gray-500">{field.description}</div>
      {field.kind === "textarea" ? (
        <textarea
          aria-label={field.label}
          value={value}
          onChange={(event) => updateValue(event.target.value)}
          rows={field.rows ?? 5}
          className="w-full rounded border border-gray-200 bg-gray-50 px-3 py-2 font-mono text-[11px] text-gray-900"
        />
      ) : (
        <input
          aria-label={field.label}
          value={value}
          onChange={(event) => updateValue(event.target.value)}
          className="w-full rounded border border-gray-200 bg-gray-50 px-3 py-2 text-[11px] text-gray-900"
        />
      )}
    </label>
  );
}
