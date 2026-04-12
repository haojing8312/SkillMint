import type { FeishuAdvancedSectionProps } from "./FeishuAdvancedSection.types";

type FeishuConnectionDetailsPanelProps = Pick<
  FeishuAdvancedSectionProps,
  | "connectionDetailSummary"
  | "connectionStatusLabel"
  | "pluginVersionLabel"
  | "currentAccountLabel"
  | "pendingPairingCount"
  | "lastEventAtLabel"
  | "recentIssueLabel"
  | "runtimeLogsLabel"
  | "retryingFeishuConnector"
  | "onRefreshFeishuSetup"
  | "onCopyDiagnostics"
>;

export function FeishuConnectionDetailsPanel({
  connectionDetailSummary,
  connectionStatusLabel,
  pluginVersionLabel,
  currentAccountLabel,
  pendingPairingCount,
  lastEventAtLabel,
  recentIssueLabel,
  runtimeLogsLabel,
  retryingFeishuConnector,
  onRefreshFeishuSetup,
  onCopyDiagnostics,
}: FeishuConnectionDetailsPanelProps) {
  return (
    <details className="rounded-lg border border-gray-200 bg-white p-4">
      <summary className="cursor-pointer text-sm font-medium text-gray-900">连接详情</summary>
      <div className="mt-2 text-xs text-gray-500">这里展示当前连接是否正常、最近一次事件，以及排查问题时最有用的诊断摘要。</div>
      <div className="mt-3 rounded-lg border border-blue-100 bg-blue-50 px-3 py-3 text-sm text-blue-900">{connectionDetailSummary}</div>
      <div className="mt-3 flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => void onRefreshFeishuSetup()}
          disabled={retryingFeishuConnector}
          className="h-8 px-3 rounded border border-gray-200 bg-white text-xs text-gray-700 hover:bg-gray-50 disabled:bg-gray-100"
        >
          {retryingFeishuConnector ? "检测中..." : "重新检测"}
        </button>
        <button
          type="button"
          onClick={() => void onCopyDiagnostics()}
          className="h-8 px-3 rounded border border-blue-200 bg-white text-xs text-blue-700 hover:bg-blue-50"
        >
          复制诊断摘要
        </button>
      </div>
      <div className="mt-3 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">当前状态</div>
          <div className="text-sm font-medium text-gray-900">{connectionStatusLabel}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">插件版本</div>
          <div className="text-sm font-medium text-gray-900">{pluginVersionLabel}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">当前接入账号</div>
          <div className="text-sm font-medium text-gray-900">{currentAccountLabel}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">待完成授权</div>
          <div className="text-sm font-medium text-gray-900">{pendingPairingCount}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2 md:col-span-2">
          <div className="text-[11px] text-gray-500">最近一次事件</div>
          <div className="text-sm font-medium text-gray-900">{lastEventAtLabel}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2 md:col-span-2">
          <div className="text-[11px] text-gray-500">最近问题</div>
          <div className="text-sm font-medium text-gray-900">{recentIssueLabel}</div>
        </div>
      </div>
      <details className="mt-3 rounded-lg border border-gray-100 bg-gray-50 p-3">
        <summary className="cursor-pointer text-xs font-medium text-gray-700">原始日志（最近 3 条）</summary>
        <div className="mt-2 text-xs text-gray-700 whitespace-pre-wrap break-all">{runtimeLogsLabel}</div>
      </details>
    </details>
  );
}
