import { useState } from "react";
import type { EmployeeCuratorReports, EmployeeCuratorSchedulerStatus } from "../../../types";

interface EmployeeCuratorReportsSectionProps {
  loading: boolean;
  error: string;
  reports: EmployeeCuratorReports | null;
  schedulerStatus?: EmployeeCuratorSchedulerStatus | null;
  schedulerError?: string;
  actionLoading?: string | null;
  onScan?: () => void | Promise<void>;
  onRun?: () => void | Promise<void>;
  onRestoreStaleSkill?: (skillId: string) => void | Promise<void>;
}

function kindLabel(kind: string) {
  switch (kind) {
    case "duplicate_memory":
      return "重复记忆";
    case "reusable_skill_candidate":
      return "可沉淀技能";
    case "low_value_debris":
      return "低价值碎片";
    case "stale_skill":
      return "Stale 技能";
    case "stale_skill_candidate":
      return "待整理技能";
    case "skill_improvement_candidate":
      return "待补全技能";
    case "pinned_skill_protected":
      return "固定技能已跳过";
    case "curator_restore":
      return "恢复技能";
    default:
      return kind || "整理建议";
  }
}

function modeLabel(mode?: string) {
  switch (mode) {
    case "run":
      return "Run";
    case "restore":
      return "Restore";
    case "scan":
      return "Scan";
    default:
      return mode || "Report";
  }
}

function severityClass(severity: string) {
  if (severity === "medium") return "border-amber-200 bg-amber-50 text-amber-700";
  if (severity === "high") return "border-red-200 bg-red-50 text-red-700";
  return "border-cyan-200 bg-cyan-50 text-cyan-700";
}

export function EmployeeCuratorReportsSection({
  loading,
  error,
  reports,
  schedulerStatus,
  schedulerError,
  actionLoading,
  onScan,
  onRun,
  onRestoreStaleSkill,
}: EmployeeCuratorReportsSectionProps) {
  const runs = reports?.runs ?? [];
  const [expandedReportId, setExpandedReportId] = useState<string | null>(null);
  const [selectedReportId, setSelectedReportId] = useState<string | null>(null);
  const latest = runs.find((run) => run.id === selectedReportId) ?? runs[0] ?? null;
  const findingCount = latest?.findings.length ?? 0;
  const restoreCandidates = latest?.restore_candidates ?? [];
  const changedTargets = latest?.changed_targets ?? [];
  const expanded = Boolean(latest && expandedReportId === latest.id);
  const reportJson = latest ? JSON.stringify(latest, null, 2) : "";
  const scanLoading = actionLoading === "scan";
  const runLoading = actionLoading === "run";
  const actionDisabled = Boolean(actionLoading);
  const autoStateLabel = schedulerStatus?.running
    ? "自动整理运行中"
    : schedulerStatus?.enabled
      ? schedulerStatus.idle
        ? "自动整理已启用"
        : "等待空闲"
      : "自动整理已暂停";
  const autoStateClass = schedulerStatus?.running
    ? "border-emerald-200 bg-emerald-50 text-emerald-700"
    : schedulerStatus?.enabled
      ? schedulerStatus.idle
        ? "border-cyan-200 bg-white text-cyan-700"
        : "border-amber-200 bg-amber-50 text-amber-700"
      : "border-gray-200 bg-white text-gray-500";
  const profileRunLabel = schedulerStatus?.profile_last_run_at
    ? `上次整理 ${schedulerStatus.profile_last_run_at}`
    : schedulerStatus?.profile_due
      ? "等待首次自动整理"
      : "已纳入自动维护";
  const lastCompletedLabel = schedulerStatus?.last_completed_at
    ? schedulerStatus.last_completed_at
    : latest?.created_at || "暂无完成记录";
  const nextCheckLabel = schedulerStatus?.next_check_at || "等待调度";
  const recentSuggestionLabel = latest
    ? `${findingCount} 个最近建议${latest.has_state_changes ? " · 有状态变更" : ""}`
    : "暂无最近建议";
  const subtitle = schedulerError
    ? "自动状态暂不可用"
    : schedulerStatus
      ? autoStateLabel
      : loading
        ? "读取中..."
        : error
          ? "暂不可用"
          : latest
            ? `${findingCount} 个最近建议`
            : "等待自动整理";

  return (
    <div className="rounded-lg border border-cyan-100 bg-cyan-50/70 p-3">
      <div className="flex items-center justify-between gap-2">
        <div>
          <div className="text-xs font-medium text-cyan-900">Curator 自动整理</div>
          <div className="mt-1 text-[11px] text-cyan-800">{subtitle}</div>
        </div>
        <div className="flex flex-wrap items-center justify-end gap-1.5">
          {schedulerStatus && (
            <span className={`rounded border px-2 py-1 text-[10px] ${autoStateClass}`}>
              {autoStateLabel}
            </span>
          )}
          <span className="rounded border border-cyan-100 bg-white px-2 py-1 text-[10px] text-cyan-700">
            {recentSuggestionLabel}
          </span>
        </div>
      </div>

      {schedulerStatus && (
        <div className="mt-3 grid gap-2 md:grid-cols-4">
          <div className="rounded border border-cyan-100 bg-white px-2 py-1.5">
            <div className="text-[10px] uppercase text-cyan-600">自动周期</div>
            <div className="mt-0.5 text-[11px] text-gray-700">
              每 {schedulerStatus.interval_minutes} 分钟
            </div>
          </div>
          <div className="rounded border border-cyan-100 bg-white px-2 py-1.5">
            <div className="text-[10px] uppercase text-cyan-600">空闲条件</div>
            <div className="mt-0.5 text-[11px] text-gray-700">
              {schedulerStatus.idle ? "当前空闲" : `${schedulerStatus.active_run_count} 个运行中`}
            </div>
          </div>
          <div className="rounded border border-cyan-100 bg-white px-2 py-1.5">
            <div className="text-[10px] uppercase text-cyan-600">上次整理</div>
            <div className="mt-0.5 truncate text-[11px] text-gray-700" title={lastCompletedLabel}>
              {lastCompletedLabel}
            </div>
          </div>
          <div className="rounded border border-cyan-100 bg-white px-2 py-1.5">
            <div className="text-[10px] uppercase text-cyan-600">下次检查</div>
            <div className="mt-0.5 truncate text-[11px] text-gray-700" title={nextCheckLabel}>
              {nextCheckLabel}
            </div>
          </div>
        </div>
      )}

      {schedulerStatus && (
        <div className="mt-2 rounded border border-cyan-100 bg-white px-2 py-1.5 text-[11px] text-cyan-800">
          {profileRunLabel}
        </div>
      )}

      <details className="mt-2 text-[10px] text-cyan-700">
        <summary className="cursor-pointer select-none">调试 / 立即执行</summary>
        <div className="mt-2 flex flex-wrap gap-1.5 rounded border border-cyan-100 bg-white p-2">
          <button
            type="button"
            data-testid="employee-curator-scan"
            disabled={actionDisabled}
            onClick={onScan}
            className="h-7 rounded border border-cyan-200 bg-white px-2 text-[10px] text-cyan-700 hover:bg-cyan-100 disabled:opacity-60"
          >
            {scanLoading ? "扫描中..." : "立即 Scan（调试）"}
          </button>
          <button
            type="button"
            data-testid="employee-curator-run"
            disabled={actionDisabled}
            onClick={onRun}
            className="h-7 rounded border border-emerald-200 bg-white px-2 text-[10px] text-emerald-700 hover:bg-emerald-50 disabled:opacity-60"
          >
            {runLoading ? "整理中..." : "立即 Run（调试）"}
          </button>
        </div>
      </details>

      {schedulerStatus?.last_error && (
        <div className="mt-2 rounded border border-amber-100 bg-white px-2 py-1.5 text-[11px] text-amber-700">
          {schedulerStatus.last_error}
        </div>
      )}

      {error ? (
        <div className="mt-3 rounded border border-red-100 bg-white px-2 py-1.5 text-[11px] text-red-600">
          {error}
        </div>
      ) : !latest ? (
        <div className="mt-3 rounded border border-cyan-100 bg-white px-2 py-2 text-[11px] text-gray-500">
          暂无 Curator 报告。WorkClaw 会在空闲时自动生成，手动 Scan/Run 可立即触发。
        </div>
      ) : (
        <div data-testid="employee-curator-report" className="mt-3 space-y-2">
          {runs.length > 1 && (
            <div className="flex flex-wrap gap-1">
              {runs.slice(0, 5).map((run) => (
                <button
                  key={run.id}
                  type="button"
                  onClick={() => setSelectedReportId(run.id)}
                  className={
                    "rounded border px-2 py-1 text-[10px] " +
                    (latest.id === run.id
                      ? "border-cyan-300 bg-white text-cyan-800"
                      : "border-cyan-100 bg-cyan-50 text-cyan-700 hover:bg-white")
                  }
                >
                  {modeLabel(run.mode)} · {run.findings.length}
                </button>
              ))}
            </div>
          )}
          <div className="rounded border border-cyan-100 bg-white px-2 py-2">
            <div className="flex flex-col gap-1 md:flex-row md:items-center md:justify-between">
              <div className="text-xs font-medium text-gray-800">{latest.summary || latest.id}</div>
              <div className="flex flex-wrap gap-1 text-[10px]">
                <span className="rounded border border-cyan-100 bg-cyan-50 px-1.5 py-0.5 text-cyan-700">
                  {modeLabel(latest.mode)}
                </span>
                {latest.has_state_changes && (
                  <span className="rounded border border-emerald-100 bg-emerald-50 px-1.5 py-0.5 text-emerald-700">
                    已变更
                  </span>
                )}
              </div>
            </div>
            {latest.created_at && (
              <div className="mt-1 text-[10px] text-gray-400">{latest.created_at}</div>
            )}
            <div className="mt-2 flex flex-wrap items-center gap-2">
              <button
                type="button"
                data-testid="employee-curator-toggle-report"
                onClick={() => setExpandedReportId(expanded ? null : latest.id)}
                className="h-6 rounded border border-cyan-200 bg-cyan-50 px-2 text-[10px] text-cyan-700 hover:bg-cyan-100"
              >
                {expanded ? "收起报告" : "展开报告"}
              </button>
            </div>
          </div>
          {expanded && (
            <div
              data-testid="employee-curator-report-detail"
              className="rounded border border-cyan-100 bg-white px-2 py-2"
            >
              <div className="text-[11px] font-medium text-cyan-900">Report JSON</div>
              <pre className="mt-2 max-h-72 overflow-auto rounded border border-gray-100 bg-gray-50 p-2 text-[10px] text-gray-700 whitespace-pre-wrap">
                {reportJson}
              </pre>
            </div>
          )}
          {changedTargets.length > 0 && (
            <div className="rounded border border-emerald-100 bg-white px-2 py-2">
              <div className="text-[11px] font-medium text-emerald-800">状态变更</div>
              <div className="mt-1 flex flex-wrap gap-1 text-[10px] text-emerald-700">
                {changedTargets.slice(0, 4).map((target) => (
                  <span
                    key={`${target.kind}-${target.target_type}-${target.target_id}-${target.restored_to}`}
                    className="rounded border border-emerald-100 bg-emerald-50 px-1.5 py-0.5"
                  >
                    {target.target_type}:{target.target_id}
                    {target.restored_to ? ` -> ${target.restored_to}` : ""}
                  </span>
                ))}
              </div>
            </div>
          )}
          {restoreCandidates.length > 0 && (
            <div className="rounded border border-blue-100 bg-white px-2 py-2">
              <div className="text-[11px] font-medium text-blue-800">可恢复</div>
              <div className="mt-2 flex flex-wrap gap-1.5">
                {restoreCandidates.slice(0, 4).map((candidate) => {
                  const isLoading = actionLoading === candidate.target_id;
                  return (
                    <button
                      key={`${candidate.target_type}-${candidate.target_id}`}
                      type="button"
                      data-testid={`employee-curator-restore-${candidate.target_id}`}
                      disabled={Boolean(actionLoading)}
                      onClick={() => onRestoreStaleSkill?.(candidate.target_id)}
                      className="rounded border border-blue-200 bg-blue-50 px-2 py-1 text-[10px] text-blue-700 hover:bg-blue-100 disabled:opacity-60"
                    >
                      {isLoading ? "恢复中..." : `恢复 ${candidate.target_id}`}
                    </button>
                  );
                })}
              </div>
            </div>
          )}
          {latest.findings.length === 0 ? (
            <div className="rounded border border-cyan-100 bg-white px-2 py-2 text-[11px] text-gray-500">
              本次未发现需要整理的条目
            </div>
          ) : (
            latest.findings.slice(0, 4).map((finding) => (
              <div
                key={`${finding.kind}-${finding.target_type}-${finding.target_id}-${finding.summary}`}
                className="rounded border border-cyan-100 bg-white px-2 py-2"
              >
                <div className="flex flex-col gap-1 md:flex-row md:items-center md:justify-between">
                  <div className="text-xs font-medium text-gray-800">{kindLabel(finding.kind)}</div>
                  <div className={`w-fit rounded border px-1.5 py-0.5 text-[10px] ${severityClass(finding.severity)}`}>
                    {finding.severity || "low"}
                  </div>
                </div>
                <div className="mt-1 text-[11px] text-gray-700">
                  {finding.summary || finding.target_id || "-"}
                </div>
                <div className="mt-1 flex flex-wrap gap-1 text-[10px] text-gray-500">
                  <span className="rounded border border-gray-100 px-1.5 py-0.5">
                    {finding.target_type}:{finding.target_id}
                  </span>
                  {finding.reversible && (
                    <span className="rounded border border-emerald-100 bg-emerald-50 px-1.5 py-0.5 text-emerald-700">
                      可回滚
                    </span>
                  )}
                </div>
                {finding.suggested_action && (
                  <div className="mt-1 text-[10px] text-gray-500">{finding.suggested_action}</div>
                )}
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
