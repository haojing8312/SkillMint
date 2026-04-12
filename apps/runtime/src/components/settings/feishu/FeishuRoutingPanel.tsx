import type { FeishuAdvancedConsoleSectionProps } from "./FeishuAdvancedConsoleSection.types";

type FeishuRoutingPanelProps = Pick<
  FeishuAdvancedConsoleSectionProps,
  "onOpenEmployees" | "feishuSetupProgress" | "feishuRoutingStatus"
>;

export function FeishuRoutingPanel({
  onOpenEmployees,
  feishuSetupProgress,
  feishuRoutingStatus,
}: FeishuRoutingPanelProps) {
  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4 space-y-3">
      <div>
        <div className="text-sm font-medium text-gray-900">接待设置</div>
        <div className="text-xs text-gray-500 mt-1">飞书接通后，还需要指定默认接待员工或配置群聊范围，消息才会稳定落到正确员工。</div>
      </div>
      <div className="rounded-lg border border-blue-100 bg-blue-50 px-3 py-3">
        <div className="text-sm font-medium text-blue-950">{feishuRoutingStatus.label}</div>
        <div className="mt-1 text-xs text-blue-900">{feishuRoutingStatus.description}</div>
      </div>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">授权状态</div>
          <div className="text-sm font-medium text-gray-900">{feishuSetupProgress?.auth_status === "approved" ? "已完成" : "待完成"}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">默认接待员工</div>
          <div className="text-sm font-medium text-gray-900">{feishuSetupProgress?.default_routing_employee_name || "未设置"}</div>
        </div>
        <div className="rounded border border-gray-100 bg-gray-50 px-3 py-2">
          <div className="text-[11px] text-gray-500">群聊范围规则</div>
          <div className="text-sm font-medium text-gray-900">{feishuSetupProgress?.scoped_routing_count ?? 0} 条</div>
        </div>
      </div>
      <div className="rounded-lg border border-blue-100 bg-blue-50 px-3 py-2 text-xs text-blue-800">
        接待员工的具体配置入口在员工详情页。完成当前接入后，请关闭设置窗口并前往员工详情中的“飞书接待”继续配置。
      </div>
      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => onOpenEmployees?.()}
          className="h-8 px-3 rounded border border-blue-200 bg-white text-xs text-blue-700 hover:bg-blue-50"
        >
          {feishuRoutingStatus.actionLabel}
        </button>
      </div>
    </div>
  );
}
