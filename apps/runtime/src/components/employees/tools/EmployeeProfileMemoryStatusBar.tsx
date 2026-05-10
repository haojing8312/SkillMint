import type { EmployeeProfileMemoryStatus } from "../../../types";

interface EmployeeProfileMemoryStatusBarProps {
  loading: boolean;
  error: string;
  status: EmployeeProfileMemoryStatus | null;
}

function sourceLabel(source?: string) {
  if (source === "profile") return "Profile Home";
  if (source === "none") return "No Memory";
  return "Unknown";
}

function sourceClass(source?: string) {
  if (source === "profile") return "border-emerald-200 bg-emerald-50 text-emerald-700";
  if (source === "none") return "border-gray-200 bg-gray-50 text-gray-600";
  return "border-red-200 bg-red-50 text-red-700";
}

export function EmployeeProfileMemoryStatusBar({
  loading,
  error,
  status,
}: EmployeeProfileMemoryStatusBarProps) {
  const activeSource = error ? "unknown" : status?.active_source || "none";
  const activePath = status?.active_source_path || status?.profile_memory_file_path || "";

  return (
    <div className="rounded-lg border border-emerald-100 bg-emerald-50/70 p-3">
      <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
        <div>
          <div className="text-xs font-medium text-emerald-900">Profile Home</div>
          <div className="mt-1 text-[11px] text-emerald-800">
            {loading
              ? "检查中..."
              : error
                ? "状态暂不可用"
                : activeSource === "profile"
                  ? "正在读取员工 Profile 记忆"
                  : "暂无可用长期记忆"}
          </div>
        </div>
        <div
          data-testid="employee-profile-memory-source"
          className={`inline-flex h-7 items-center rounded border px-2.5 text-[11px] font-medium ${sourceClass(
            activeSource,
          )}`}
        >
          {sourceLabel(activeSource)}
        </div>
      </div>
      <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
        <div className="rounded border border-emerald-100 bg-white px-2 py-1.5">
          <div className="text-[10px] uppercase text-gray-400">Skill</div>
          <div data-testid="employee-profile-memory-skill" className="truncate text-[11px] text-gray-700">
            {status?.skill_id || "-"}
          </div>
        </div>
        <div className="rounded border border-emerald-100 bg-white px-2 py-1.5">
          <div className="text-[10px] uppercase text-gray-400">Profile File</div>
          <div className="text-[11px] text-gray-700">
            {status?.profile_memory_file_exists ? "已存在" : "未创建"}
          </div>
        </div>
      </div>
      {activePath && (
        <div className="mt-2 truncate text-[10px] text-gray-500" title={activePath}>
          {activePath}
        </div>
      )}
    </div>
  );
}
