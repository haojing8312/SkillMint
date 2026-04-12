import type { FeishuAdvancedConsoleSectionProps } from "./FeishuAdvancedConsoleSection.types";

type FeishuEnvironmentPanelProps = Pick<
  FeishuAdvancedConsoleSectionProps,
  "feishuEnvironmentStatus" | "getFeishuEnvironmentLabel"
>;

export function FeishuEnvironmentPanel({
  feishuEnvironmentStatus,
  getFeishuEnvironmentLabel,
}: FeishuEnvironmentPanelProps) {
  const requiredNodeMajor = feishuEnvironmentStatus?.required_node_major ?? 22;
  const nodeVersionSupported =
    feishuEnvironmentStatus?.node_version_supported ?? Boolean(feishuEnvironmentStatus?.node_available);
  const nodeReady = Boolean(feishuEnvironmentStatus?.node_available) && nodeVersionSupported;
  const nodeVersionLabel = feishuEnvironmentStatus?.node_version
    ? nodeVersionSupported
      ? feishuEnvironmentStatus.node_version
      : `${feishuEnvironmentStatus.node_version} · 需要 >= v${requiredNodeMajor}`
    : `请安装 Node.js ${requiredNodeMajor} LTS`;
  const environmentBlockingHint =
    feishuEnvironmentStatus?.error ||
    `请先安装或升级到 Node.js ${requiredNodeMajor} LTS，完成后重新打开 WorkClaw 或回到这里点击“重新检测”。`;

  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4 space-y-3">
      <div>
        <div className="text-sm font-medium text-gray-900">检查运行环境</div>
        <div className="text-xs text-gray-500 mt-1">飞书官方插件使用宿主机自己的 Node.js；当前要求 Node.js {requiredNodeMajor}+。</div>
      </div>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-3">
          <div className="text-[11px] text-gray-500">Node.js</div>
          <div className="mt-1 text-sm font-medium text-gray-900">
            {getFeishuEnvironmentLabel(
              nodeReady,
              Boolean(feishuEnvironmentStatus?.node_available) ? "版本过低" : "未检测到",
            )}
          </div>
          <div className="mt-1 text-[11px] text-gray-500">{nodeVersionLabel}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-3">
          <div className="text-[11px] text-gray-500">npm</div>
          <div className="mt-1 text-sm font-medium text-gray-900">
            {getFeishuEnvironmentLabel(Boolean(feishuEnvironmentStatus?.npm_available), "未检测到")}
          </div>
          <div className="mt-1 text-[11px] text-gray-500">{feishuEnvironmentStatus?.npm_version || "安装 Node.js 后通常会一起提供"}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-3">
          <div className="text-[11px] text-gray-500">飞书连接组件运行条件</div>
          <div className="mt-1 text-sm font-medium text-gray-900">
            {feishuEnvironmentStatus?.can_start_runtime ? "已准备好" : "暂未满足"}
          </div>
          <div className="mt-1 text-[11px] text-gray-500">{feishuEnvironmentStatus?.error || "完成环境检查后即可继续后续步骤"}</div>
        </div>
      </div>
      {!feishuEnvironmentStatus?.can_start_runtime ? (
        <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800">
          当前电脑还没有满足飞书连接所需环境。{environmentBlockingHint}
        </div>
      ) : null}
    </div>
  );
}
