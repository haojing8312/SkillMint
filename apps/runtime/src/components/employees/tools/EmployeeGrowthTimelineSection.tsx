import type { EmployeeGrowthTimeline } from "../../../types";

interface EmployeeGrowthTimelineSectionProps {
  loading: boolean;
  error: string;
  timeline: EmployeeGrowthTimeline | null;
}

function eventLabel(eventType: string) {
  switch (eventType) {
    case "skill_create":
      return "创建技能";
    case "skill_patch":
      return "优化技能";
    case "skill_archive":
      return "归档技能";
    case "skill_restore":
      return "恢复技能";
    case "skill_delete":
      return "删除技能";
    case "skill_rollback":
      return "回滚技能";
    case "skill_reset":
      return "重置技能";
    case "memory_add":
      return "写入记忆";
    case "memory_replace":
      return "更新记忆";
    case "memory_remove":
      return "删除记忆";
    case "memory_rollback":
      return "回滚记忆";
    case "curator_scan":
      return "整理扫描";
    case "user_correction":
      return "用户纠正";
    default:
      return eventType || "成长事件";
  }
}

function evidenceSummary(evidence: Record<string, unknown>) {
  const versionId = typeof evidence.version_id === "string" ? evidence.version_id : "";
  const rollbackTo =
    typeof evidence.rollback_to_version_id === "string" ? evidence.rollback_to_version_id : "";
  const resetTo =
    typeof evidence.reset_to_version_id === "string" ? evidence.reset_to_version_id : "";
  return [versionId && `version ${versionId}`, rollbackTo && `rollback ${rollbackTo}`, resetTo && `reset ${resetTo}`]
    .filter(Boolean)
    .join(" · ");
}

function evidenceText(evidence: Record<string, unknown>, key: "content_preview" | "diff_summary") {
  const direct = evidence[key];
  if (typeof direct === "string" && direct.trim()) return direct.trim();
  const memoryVersion = evidence.memory_version;
  if (memoryVersion && typeof memoryVersion === "object" && !Array.isArray(memoryVersion)) {
    const nested = (memoryVersion as Record<string, unknown>)[key];
    if (typeof nested === "string" && nested.trim()) return nested.trim();
  }
  return "";
}

function memoryEvidenceSummary(eventType: string, evidence: Record<string, unknown>) {
  if (!eventType.startsWith("memory_") && eventType !== "user_correction") return "";
  return evidenceText(evidence, "content_preview") || evidenceText(evidence, "diff_summary");
}

export function EmployeeGrowthTimelineSection({
  loading,
  error,
  timeline,
}: EmployeeGrowthTimelineSectionProps) {
  const events = timeline?.events ?? [];

  return (
    <div className="rounded-lg border border-sky-100 bg-sky-50/70 p-3">
      <div className="flex items-center justify-between gap-2">
        <div>
          <div className="text-xs font-medium text-sky-900">成长记录</div>
          <div className="mt-1 text-[11px] text-sky-800">
            {loading ? "读取中..." : error ? "暂不可用" : `${events.length} 条最近记录`}
          </div>
        </div>
        {timeline?.profile_id && (
          <div
            data-testid="employee-growth-profile"
            className="max-w-[220px] truncate rounded border border-sky-200 bg-white px-2 py-1 text-[10px] text-sky-700"
            title={timeline.profile_id}
          >
            Profile Memory OS
          </div>
        )}
      </div>

      {error ? (
        <div className="mt-3 rounded border border-red-100 bg-white px-2 py-1.5 text-[11px] text-red-600">
          {error}
        </div>
      ) : events.length === 0 ? (
        <div className="mt-3 rounded border border-sky-100 bg-white px-2 py-2 text-[11px] text-gray-500">
          暂无成长记录
        </div>
      ) : (
        <div data-testid="employee-growth-events" className="mt-3 space-y-2">
          {events.map((event) => {
            const auditEvidence = evidenceSummary(event.evidence_json);
            const memorySummary = memoryEvidenceSummary(event.event_type, event.evidence_json);
            const displaySummary =
              memorySummary || event.display_summary?.trim() || event.summary?.trim() || event.target_label || eventLabel(event.event_type);
            const targetLabel = event.target_label?.trim();
            const evidenceLabel = event.evidence_label?.trim();
            const sessionLabel = event.session_title?.trim();
            return (
              <div key={event.id} className="rounded border border-sky-100 bg-white px-2 py-2">
                <div className="flex flex-col gap-1 md:flex-row md:items-center md:justify-between">
                  <div className="text-xs font-medium text-gray-800">{eventLabel(event.event_type)}</div>
                  <div className="text-[10px] text-gray-400">{event.created_at}</div>
                </div>
                <div className="mt-1 text-[11px] text-gray-700">
                  {displaySummary}
                </div>
                <div className="mt-1 flex flex-wrap gap-1 text-[10px] text-gray-500">
                  {sessionLabel && (
                    <span className="rounded border border-gray-100 px-1.5 py-0.5" title={event.session_id}>
                      来源：{sessionLabel}
                    </span>
                  )}
                  {targetLabel && (
                    <span className="rounded border border-gray-100 px-1.5 py-0.5">
                      {targetLabel}
                    </span>
                  )}
                  {evidenceLabel && (
                    <span className="rounded border border-gray-100 px-1.5 py-0.5">
                      {evidenceLabel}
                    </span>
                  )}
                </div>
                {(event.session_id || event.target_id || auditEvidence) && (
                  <details className="mt-2 text-[10px] text-gray-400">
                    <summary className="cursor-pointer select-none">审计详情</summary>
                    <div className="mt-1 space-y-1 rounded border border-gray-100 bg-gray-50 p-2">
                      {event.session_id && <div>session: {event.session_id}</div>}
                      {event.target_id && <div>target: {event.target_id}</div>}
                      {auditEvidence && <div>{auditEvidence}</div>}
                    </div>
                  </details>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
