interface ChatEmployeeAssistantContextProps {
  employeeAssistantContext?: {
    mode: "create" | "update";
    employeeName?: string;
    employeeCode?: string;
  };
}

export function ChatEmployeeAssistantContext({
  employeeAssistantContext,
}: ChatEmployeeAssistantContextProps) {
  if (!employeeAssistantContext) {
    return null;
  }

  return (
    <div className="space-y-3">
      <div
        data-testid="chat-employee-assistant-context"
        className="rounded-xl border border-blue-200 bg-blue-50 px-4 py-2 text-xs text-blue-800"
      >
        {employeeAssistantContext.mode === "update"
          ? `正在修改：${employeeAssistantContext.employeeName || "目标员工"}${
              employeeAssistantContext.employeeCode ? `（${employeeAssistantContext.employeeCode}）` : ""
            }`
          : "正在创建：新智能体员工"}
      </div>
      {employeeAssistantContext.mode === "create" && (
        <div className="max-w-[80%] rounded-2xl border border-blue-100 bg-white px-5 py-4 text-sm text-slate-700 shadow-sm">
          我会先问 1-2 个关键问题，再给出配置草案，确认后执行创建。
        </div>
      )}
    </div>
  );
}
