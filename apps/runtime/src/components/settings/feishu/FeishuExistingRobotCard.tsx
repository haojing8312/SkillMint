import type { FeishuAdvancedConsoleSectionProps } from "./FeishuAdvancedConsoleSection.types";

type FeishuExistingRobotCardProps = Pick<
  FeishuAdvancedConsoleSectionProps,
  | "feishuConnectorSettings"
  | "onUpdateFeishuConnectorSettings"
  | "feishuCredentialProbe"
  | "validatingFeishuCredentials"
  | "savingFeishuConnector"
  | "handleValidateFeishuCredentials"
  | "handleSaveFeishuConnector"
>;

export function FeishuExistingRobotCard({
  feishuConnectorSettings,
  onUpdateFeishuConnectorSettings,
  feishuCredentialProbe,
  validatingFeishuCredentials,
  savingFeishuConnector,
  handleValidateFeishuCredentials,
  handleSaveFeishuConnector,
}: FeishuExistingRobotCardProps) {
  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4 space-y-3">
      <div>
        <div className="text-sm font-medium text-gray-900">绑定已有机器人</div>
        <div className="text-xs text-gray-500 mt-1">这里只需要填写已有机器人的 App ID 和 App Secret；当前版本不再展示 webhook 相关配置。</div>
      </div>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
        <label className="space-y-1">
          <div className="text-[11px] font-medium text-gray-700">App ID</div>
          <input
            value={feishuConnectorSettings.app_id}
            onChange={(event) => onUpdateFeishuConnectorSettings({ app_id: event.target.value })}
            className="w-full rounded border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-900"
            placeholder="cli_xxx"
          />
        </label>
        <label className="space-y-1">
          <div className="text-[11px] font-medium text-gray-700">App Secret</div>
          <input
            type="password"
            value={feishuConnectorSettings.app_secret}
            onChange={(event) => onUpdateFeishuConnectorSettings({ app_secret: event.target.value })}
            className="w-full rounded border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-900"
            placeholder="填写机器人的 App Secret"
          />
        </label>
      </div>
      {feishuCredentialProbe?.ok ? (
        <div className="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-800">
          已识别机器人
          {feishuCredentialProbe.bot_name ? `：${feishuCredentialProbe.bot_name}` : ""}。
          {feishuCredentialProbe.bot_open_id ? ` open_id：${feishuCredentialProbe.bot_open_id}` : ""}
        </div>
      ) : null}
      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => void handleValidateFeishuCredentials()}
          disabled={validatingFeishuCredentials}
          className="h-8 px-3 rounded border border-blue-200 bg-white text-xs text-blue-700 hover:bg-blue-50 disabled:bg-gray-100"
        >
          {validatingFeishuCredentials ? "验证中..." : "验证机器人信息"}
        </button>
        <button
          type="button"
          onClick={() => void handleSaveFeishuConnector()}
          disabled={savingFeishuConnector}
          className="h-8 px-3 rounded bg-blue-600 text-xs text-white hover:bg-blue-700 disabled:bg-blue-300"
        >
          {savingFeishuConnector ? "保存中..." : "保存并继续"}
        </button>
      </div>
    </div>
  );
}
