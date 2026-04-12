import type { FeishuSetupSummary, FeishuRoutingStatus } from "./FeishuSettingsSection.types";
import type { FeishuPluginEnvironmentStatus, FeishuSetupProgress } from "../../../types";

type FeishuSettingsOverviewProps = {
  feishuSetupSummary: FeishuSetupSummary;
  feishuEnvironmentStatus: FeishuPluginEnvironmentStatus | null;
  feishuSetupProgress: FeishuSetupProgress | null;
  feishuRoutingStatus: FeishuRoutingStatus;
};

export function FeishuSettingsOverview({
  feishuSetupSummary,
  feishuEnvironmentStatus,
  feishuSetupProgress,
  feishuRoutingStatus,
}: FeishuSettingsOverviewProps) {
  return (
    <div className="rounded-xl border border-blue-200 bg-blue-50 p-4">
      <div className="text-base font-medium text-blue-950">{feishuSetupSummary.title}</div>
      <div className="mt-1 text-sm text-blue-900">{feishuSetupSummary.description}</div>
      <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-4">
        <div className="rounded border border-blue-100 bg-white/80 px-3 py-2">
          <div className="text-[11px] text-blue-700">运行环境</div>
          <div className="text-sm font-medium text-gray-900">
            {feishuEnvironmentStatus?.can_start_runtime ? "已准备好" : "待检查"}
          </div>
        </div>
        <div className="rounded border border-blue-100 bg-white/80 px-3 py-2">
          <div className="text-[11px] text-blue-700">机器人信息</div>
          <div className="text-sm font-medium text-gray-900">
            {feishuSetupProgress?.credentials_configured ? "已填写" : "未填写"}
          </div>
        </div>
        <div className="rounded border border-blue-100 bg-white/80 px-3 py-2">
          <div className="text-[11px] text-blue-700">连接组件</div>
          <div className="text-sm font-medium text-gray-900">
            {feishuSetupProgress?.plugin_installed ? "已安装" : "未安装"}
          </div>
        </div>
        <div className="rounded border border-blue-100 bg-white/80 px-3 py-2">
          <div className="text-[11px] text-blue-700">授权与接待</div>
          <div className="text-sm font-medium text-gray-900">{feishuRoutingStatus.label}</div>
        </div>
      </div>
      <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
        <div className="rounded border border-blue-100 bg-white/80 px-3 py-2">
          <div className="text-sm font-medium text-gray-900">飞书接入概览</div>
          <div className="mt-1 text-xs text-gray-600">查看飞书连接是否已启动并可接收事件。</div>
        </div>
        <div className="rounded border border-blue-100 bg-white/80 px-3 py-2">
          <div className="text-sm font-medium text-gray-900">员工关联入口</div>
          <div className="mt-1 text-xs text-gray-600">先完成飞书连接，再到员工详情中指定谁来接待飞书消息。</div>
          <div className="mt-1 text-xs text-gray-600">
            飞书连接成功后，请前往员工详情中的“飞书接待”配置默认接待员工或指定群聊范围。
          </div>
        </div>
      </div>
    </div>
  );
}
