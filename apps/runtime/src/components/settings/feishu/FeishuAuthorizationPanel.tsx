import { Fragment } from "react";
import type { FeishuAdvancedConsoleSectionProps } from "./FeishuAdvancedConsoleSection.types";

type FeishuAuthorizationPanelProps = Pick<
  FeishuAdvancedConsoleSectionProps,
  | "feishuSetupProgress"
  | "officialFeishuRuntimeStatus"
  | "retryingFeishuConnector"
  | "installingOfficialFeishuPlugin"
  | "feishuInstallerSession"
  | "feishuInstallerInput"
  | "onUpdateFeishuInstallerInput"
  | "feishuInstallerBusy"
  | "feishuInstallerStartingMode"
  | "feishuPairingActionLoading"
  | "pendingFeishuPairingCount"
  | "pendingFeishuPairingRequest"
  | "feishuAuthorizationInlineError"
  | "feishuOnboardingHeaderStep"
  | "feishuInstallerDisplayMode"
  | "feishuInstallerStartupHint"
  | "feishuAuthorizationAction"
  | "formatCompactDateTime"
  | "handleInstallAndStartFeishuConnector"
  | "handleRefreshFeishuSetup"
  | "handleResolveFeishuPairingRequest"
  | "handleStartFeishuInstaller"
  | "handleStopFeishuInstallerSession"
  | "handleSendFeishuInstallerInput"
>;

export function FeishuAuthorizationPanel({
  feishuSetupProgress,
  officialFeishuRuntimeStatus,
  retryingFeishuConnector,
  installingOfficialFeishuPlugin,
  feishuInstallerSession,
  feishuInstallerInput,
  onUpdateFeishuInstallerInput,
  feishuInstallerBusy,
  feishuInstallerStartingMode,
  feishuPairingActionLoading,
  pendingFeishuPairingCount,
  pendingFeishuPairingRequest,
  feishuAuthorizationInlineError,
  feishuOnboardingHeaderStep,
  feishuInstallerDisplayMode,
  feishuInstallerStartupHint,
  feishuAuthorizationAction,
  formatCompactDateTime,
  handleInstallAndStartFeishuConnector,
  handleRefreshFeishuSetup,
  handleResolveFeishuPairingRequest,
  handleStartFeishuInstaller,
  handleStopFeishuInstallerSession,
  handleSendFeishuInstallerInput,
}: FeishuAuthorizationPanelProps) {
  return (
    <div data-testid="feishu-authorization-step" className="rounded-lg border border-gray-200 bg-white p-4 space-y-3">
      <div>
        <div className="text-sm font-medium text-gray-900">
          {pendingFeishuPairingCount > 0 ? "批准飞书接入请求" : "完成飞书授权"}
        </div>
        <div className="text-xs text-gray-500 mt-1">
          {pendingFeishuPairingCount > 0
            ? "飞书里的机器人已经发来了接入请求。请先在这里批准这次接入，再继续后续配置。"
            : "安装并启动后，请回到飞书中的机器人会话按提示完成授权，然后回到这里刷新状态。"}
        </div>
      </div>
      <div className="rounded-lg border border-gray-100 bg-gray-50 px-3 py-3 text-xs text-gray-700 space-y-1">
        {pendingFeishuPairingCount > 0 ? (
          <>
            <div>1. 飞书里已经生成了 pairing request</div>
            <div>2. 在这里点击“批准这次接入”</div>
            <div>3. 批准后再继续配置接待员工</div>
          </>
        ) : (
          <>
            <div>1. 在飞书中打开机器人会话</div>
            <div>2. 按提示完成授权</div>
            <div>3. 如果机器人提示 access not configured，下一步回来批准接入请求</div>
          </>
        )}
      </div>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">连接组件</div>
          <div className="text-sm font-medium text-gray-900">{feishuSetupProgress?.plugin_installed ? "已安装" : "未安装"}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">运行状态</div>
          <div className="text-sm font-medium text-gray-900">{officialFeishuRuntimeStatus?.running ? "运行中" : "未启动"}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">授权状态</div>
          <div className="text-sm font-medium text-gray-900">
            {pendingFeishuPairingCount > 0
              ? "待批准接入"
              : feishuSetupProgress?.auth_status === "approved"
                ? "已完成"
                : "待完成"}
          </div>
        </div>
      </div>
      {pendingFeishuPairingRequest ? (
        <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-3 text-xs text-amber-900 space-y-1">
          <div>发送者：{pendingFeishuPairingRequest.sender_id}</div>
          <div>Pairing Code：{pendingFeishuPairingRequest.code || "未返回"}</div>
          <div>发起时间：{formatCompactDateTime(pendingFeishuPairingRequest.created_at)}</div>
        </div>
      ) : null}
      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => void handleInstallAndStartFeishuConnector()}
          disabled={retryingFeishuConnector || installingOfficialFeishuPlugin}
          className="h-8 px-3 rounded bg-indigo-600 text-xs text-white hover:bg-indigo-700 disabled:bg-indigo-300"
        >
          {retryingFeishuConnector || installingOfficialFeishuPlugin
            ? feishuAuthorizationAction.busyLabel
            : feishuAuthorizationAction.label}
        </button>
        <button
          type="button"
          onClick={() => void handleRefreshFeishuSetup()}
          disabled={retryingFeishuConnector}
          className="h-8 px-3 rounded border border-gray-200 bg-white text-xs text-gray-700 hover:bg-gray-50 disabled:bg-gray-100"
        >
          刷新授权状态
        </button>
        {pendingFeishuPairingRequest ? (
          <Fragment>
            <button
              type="button"
              onClick={() => void handleResolveFeishuPairingRequest(pendingFeishuPairingRequest.id, "approve")}
              disabled={feishuPairingActionLoading !== null}
              className="h-8 px-3 rounded bg-amber-600 text-xs text-white hover:bg-amber-700 disabled:bg-amber-300"
            >
              {feishuPairingActionLoading === "approve" ? "批准中..." : "批准这次接入"}
            </button>
            <button
              type="button"
              onClick={() => void handleResolveFeishuPairingRequest(pendingFeishuPairingRequest.id, "deny")}
              disabled={feishuPairingActionLoading !== null}
              className="h-8 px-3 rounded border border-red-200 bg-white text-xs text-red-700 hover:bg-red-50 disabled:bg-gray-100"
            >
              {feishuPairingActionLoading === "deny" ? "拒绝中..." : "拒绝这次接入"}
            </button>
          </Fragment>
        ) : null}
        <button
          type="button"
          onClick={() => void handleStartFeishuInstaller("create")}
          disabled={feishuInstallerBusy}
          className="h-8 px-3 rounded border border-indigo-200 bg-white text-xs text-indigo-700 hover:bg-indigo-50 disabled:bg-gray-100"
        >
          {feishuInstallerBusy && feishuInstallerStartingMode === "create" ? "启动中..." : "新建机器人向导（高级）"}
        </button>
      </div>
      {feishuAuthorizationInlineError && feishuOnboardingHeaderStep !== "authorize" ? (
        <div className="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">
          {feishuAuthorizationInlineError}
        </div>
      ) : null}
      <details
        className="rounded-lg border border-gray-100 bg-gray-50 p-3"
        open={feishuInstallerSession.running || feishuInstallerSession.recent_output.length > 0}
      >
        <summary className="cursor-pointer text-xs font-medium text-gray-700">查看安装向导输出</summary>
        <div className="mt-3 space-y-3">
          <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
            <div className="rounded border border-gray-200 bg-white px-3 py-2">
              <div className="text-[11px] text-gray-500">向导状态</div>
              <div className="text-sm font-medium text-gray-900">
                {feishuInstallerBusy && feishuInstallerStartingMode ? "正在启动" : feishuInstallerSession.running ? "运行中" : "未运行"}
              </div>
            </div>
            <div className="rounded border border-gray-200 bg-white px-3 py-2">
              <div className="text-[11px] text-gray-500">当前模式</div>
              <div className="text-sm font-medium text-gray-900">
                {feishuInstallerDisplayMode === "create"
                  ? "新建机器人"
                  : feishuInstallerDisplayMode === "link"
                    ? "绑定已有机器人"
                    : "未启动"}
              </div>
            </div>
            <div className="rounded border border-gray-200 bg-white px-3 py-2">
              <div className="text-[11px] text-gray-500">提示</div>
              <div className="text-sm font-medium text-gray-900">
                {feishuInstallerStartupHint || feishuInstallerSession.prompt_hint || "暂无"}
              </div>
            </div>
          </div>
          <div className="rounded-lg border border-gray-900 bg-[#050816] px-3 py-3 text-xs text-gray-100">
            <pre className="max-h-72 overflow-auto whitespace-pre-wrap break-all font-mono">
              {feishuInstallerSession.recent_output.length > 0
                ? feishuInstallerSession.recent_output.join("\n")
                : feishuInstallerStartupHint || "暂无安装向导输出"}
            </pre>
          </div>
          <div className="flex flex-col gap-2 md:flex-row">
            <input
              value={feishuInstallerInput}
              onChange={(event) => onUpdateFeishuInstallerInput(event.target.value)}
              placeholder="需要时手动输入，例如 App ID、App Secret 或回车"
              className="flex-1 rounded border border-gray-200 bg-white px-3 py-2 text-xs text-gray-900"
            />
            <button
              type="button"
              onClick={() => void handleSendFeishuInstallerInput()}
              disabled={feishuInstallerBusy || !feishuInstallerInput.trim()}
              className="h-9 px-3 rounded border border-gray-200 bg-white text-xs text-gray-700 hover:bg-gray-50 disabled:bg-gray-100"
            >
              发送输入
            </button>
            <button
              type="button"
              onClick={() => void handleStopFeishuInstallerSession()}
              disabled={feishuInstallerBusy || !feishuInstallerSession.running}
              className="h-9 px-3 rounded border border-red-200 bg-white text-xs text-red-700 hover:bg-red-50 disabled:bg-gray-100"
            >
              停止向导
            </button>
          </div>
          <div className="text-[11px] text-gray-500">
            即使本机存在旧 OpenClaw 配置，当前向导也只使用 WorkClaw 内置受控兼容 shim，不会写入外部旧 OpenClaw 配置。
          </div>
        </div>
      </details>
    </div>
  );
}
