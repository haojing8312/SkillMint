import { useEffect, useMemo, useState } from "react";
import type {
  SkillOsIndexEntry,
  SkillOsVersionEntry,
  SkillOsView,
} from "../../../types";

interface EmployeeSkillOsSectionProps {
  authorizedSkillIds: string[];
  fallbackSkillNames: Map<string, string>;
  index: SkillOsIndexEntry[];
  loading: boolean;
  error: string;
  selectedSkillId: string | null;
  selectedView: SkillOsView | null;
  selectedVersions: SkillOsVersionEntry[];
  detailLoading: boolean;
  actionLoading: "patch" | "pin" | "reset" | "rollback" | "archive" | "restore" | "delete" | null;
  onSelectSkill: (skillId: string) => void;
  onRefresh: () => void | Promise<void>;
  onRequestPatch: (skillId: string, content: string) => void;
  onSetPinned: (pinned: boolean) => void;
  onRequestReset: (skillId: string) => void;
  onRequestRollback: (version: SkillOsVersionEntry) => void;
  onRequestArchive: (skillId: string) => void;
  onRequestRestore: (skillId: string) => void;
  onRequestDelete: (skillId: string) => void;
}

function sourceLabel(entry: SkillOsIndexEntry): string {
  if (entry.source.immutable_content) return ".skillpack · 只读";
  if (entry.source.canonical === "preset") return "Preset · 可进化";
  if (entry.source.canonical === "agent_created") return "Agent-created · 可进化";
  if (entry.source.canonical === "local") return "Local · 可进化";
  return `${entry.source.canonical || entry.source.raw_source_type || "unknown"}`;
}

function capabilityLabel(entry: SkillOsIndexEntry): string {
  const caps = entry.capabilities;
  if (entry.source.immutable_content) return "受保护，不会被 patch/reset/curator 修改";
  const parts = [];
  if (caps.can_patch) parts.push("patch");
  if (caps.can_reset) parts.push("reset");
  if (caps.can_archive) parts.push("archive");
  if (parts.length === 0) return "只读";
  return parts.join(" / ");
}

function renderToolsets(entry: SkillOsIndexEntry): string {
  const policy = entry.toolset_policy;
  const parts = [
    ...policy.requires_toolsets.map((item) => `requires:${item}`),
    ...policy.optional_toolsets.map((item) => `optional:${item}`),
    ...policy.denied_toolsets.map((item) => `denied:${item}`),
    ...policy.unknown_toolsets.map((item) => `unknown:${item}`),
  ];
  return parts.length > 0 ? parts.join(" · ") : "未声明";
}

function versionActionLabel(action: string): string {
  switch (action) {
    case "create":
      return "创建";
    case "patch":
      return "修改";
    case "archive":
      return "归档";
    case "restore":
      return "恢复";
    case "delete":
      return "删除";
    case "rollback":
      return "回滚";
    case "reset":
      return "重置";
    default:
      return action || "变更";
  }
}

function lineDiff(before: string, after: string): string {
  if (before === after) return "";
  const beforeLines = before.split(/\r?\n/);
  const afterLines = after.split(/\r?\n/);
  const maxLength = Math.max(beforeLines.length, afterLines.length);
  const out: string[] = [];
  for (let index = 0; index < maxLength; index += 1) {
    const left = beforeLines[index];
    const right = afterLines[index];
    if (left === right) continue;
    if (left !== undefined) out.push(`-${left}`);
    if (right !== undefined) out.push(`+${right}`);
  }
  return out.join("\n");
}

export function EmployeeSkillOsSection({
  authorizedSkillIds,
  fallbackSkillNames,
  index,
  loading,
  error,
  selectedSkillId,
  selectedView,
  selectedVersions,
  detailLoading,
  actionLoading,
  onSelectSkill,
  onRefresh,
  onRequestPatch,
  onSetPinned,
  onRequestReset,
  onRequestRollback,
  onRequestArchive,
  onRequestRestore,
  onRequestDelete,
}: EmployeeSkillOsSectionProps) {
  const indexById = new Map(index.map((entry) => [entry.skill_id, entry]));
  const rows = authorizedSkillIds.map((skillId) => {
    const entry = indexById.get(skillId);
    return {
      skillId,
      entry,
      name: entry?.name || fallbackSkillNames.get(skillId) || skillId,
    };
  });
  const selectedEntry =
    selectedView?.entry || (selectedSkillId ? indexById.get(selectedSkillId) : undefined);
  const [editing, setEditing] = useState(false);
  const [draftContent, setDraftContent] = useState("");
  const sourceContent = selectedView?.content ?? "";
  const diff = useMemo(() => lineDiff(sourceContent, draftContent), [draftContent, sourceContent]);
  const canPatch = Boolean(selectedEntry?.capabilities.can_patch && !selectedEntry.source.immutable_content);
  const isArchived = selectedEntry?.lifecycle_state === "archived";
  const canArchive = Boolean(
    selectedEntry?.capabilities.can_archive &&
      !selectedEntry.source.immutable_content &&
      !isArchived,
  );
  const canRestore = Boolean(
    selectedEntry?.capabilities.can_archive &&
      !selectedEntry.source.immutable_content &&
      isArchived,
  );
  const canDelete = Boolean(
    selectedEntry?.capabilities.can_agent_delete &&
      !selectedEntry.source.immutable_content,
  );

  useEffect(() => {
    setEditing(false);
    setDraftContent(sourceContent);
  }, [selectedSkillId, sourceContent]);

  return (
    <div className="rounded-lg border border-gray-200 p-3 space-y-3" data-testid="employee-skill-os">
      <div className="flex items-center justify-between gap-2">
        <div>
          <div className="text-sm font-semibold text-gray-900">Skills</div>
          <div className="text-[11px] text-gray-500">Profile skill library · lifecycle · versions</div>
        </div>
        <button
          type="button"
          onClick={() => onRefresh()}
          disabled={loading}
          className="h-8 px-3 rounded border border-blue-200 hover:bg-blue-50 text-blue-700 text-xs disabled:opacity-60"
        >
          刷新
        </button>
      </div>

      {error && <div className="rounded border border-red-100 bg-red-50 p-2 text-xs text-red-600">{error}</div>}

      {loading ? (
        <div className="text-xs text-gray-500">正在加载 Skill OS...</div>
      ) : rows.length === 0 ? (
        <div className="rounded border border-dashed border-gray-300 p-3 text-xs text-gray-500">
          当前员工未绑定专属技能，运行时会使用默认通用能力。
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2" data-testid="employee-skill-os-list">
          {rows.map(({ skillId, entry, name }) => {
            const active = selectedSkillId === skillId;
            return (
              <button
                key={skillId}
                type="button"
                onClick={() => onSelectSkill(skillId)}
                className={
                  "text-left rounded border p-2 text-xs " +
                  (active ? "border-blue-300 bg-blue-50" : "border-gray-200 bg-white hover:bg-gray-50")
                }
              >
                <div className="font-medium text-gray-900 truncate">{name}</div>
                <div className="text-[11px] text-gray-500 truncate">{skillId}</div>
                <div className="mt-1 flex flex-wrap gap-1">
                  <span className="rounded border border-gray-200 px-1.5 py-0.5 text-[10px] text-gray-600">
                    {entry ? sourceLabel(entry) : "未进入 Skill OS index"}
                  </span>
                  {entry && (
                    <span className="rounded border border-emerald-100 bg-emerald-50 px-1.5 py-0.5 text-[10px] text-emerald-700">
                      {capabilityLabel(entry)}
                    </span>
                  )}
                </div>
              </button>
            );
          })}
        </div>
      )}

      {selectedSkillId && (
        <div className="rounded border border-gray-100 p-3 space-y-2" data-testid="employee-skill-os-detail">
          <div className="flex items-start justify-between gap-2">
            <div>
              <div className="text-xs font-semibold text-gray-800">
                {selectedEntry?.name || fallbackSkillNames.get(selectedSkillId) || selectedSkillId}
              </div>
              <div className="text-[11px] text-gray-500 break-all">{selectedSkillId}</div>
            </div>
            {selectedEntry && (
              <div className="flex flex-wrap justify-end gap-1">
                <span className="rounded border border-gray-200 px-2 py-0.5 text-[10px] text-gray-600">
                  {sourceLabel(selectedEntry)}
                </span>
                <span className="rounded border border-gray-200 px-2 py-0.5 text-[10px] text-gray-600">
                  {selectedEntry.lifecycle_state || "active"}
                </span>
                {selectedEntry.usage.pinned && (
                  <span className="rounded border border-amber-200 bg-amber-50 px-2 py-0.5 text-[10px] text-amber-700">
                    pinned
                  </span>
                )}
              </div>
            )}
          </div>

          {selectedEntry ? (
            <>
              <div className="flex flex-wrap items-center gap-2">
                <button
                  type="button"
                  data-testid="employee-skill-os-pin"
                  disabled={Boolean(actionLoading)}
                  onClick={() => onSetPinned(!selectedEntry.usage.pinned)}
                  className="h-8 px-3 rounded border border-gray-200 bg-white hover:bg-gray-50 text-gray-700 text-xs disabled:opacity-50"
                >
                  {actionLoading === "pin"
                    ? "更新中..."
                    : selectedEntry.usage.pinned
                      ? "取消固定"
                      : "固定"}
                </button>
                <button
                  type="button"
                  data-testid="employee-skill-os-edit"
                  disabled={!canPatch || Boolean(actionLoading)}
                  onClick={() => {
                    setDraftContent(sourceContent);
                    setEditing(true);
                  }}
                  className="h-8 px-3 rounded border border-blue-200 bg-blue-50 hover:bg-blue-100 text-blue-700 text-xs disabled:opacity-50"
                >
                  编辑
                </button>
                <button
                  type="button"
                  data-testid="employee-skill-os-reset"
                  disabled={!selectedEntry.capabilities.can_reset || Boolean(actionLoading)}
                  onClick={() => onRequestReset(selectedEntry.skill_id)}
                  className="h-8 px-3 rounded border border-amber-200 bg-amber-50 hover:bg-amber-100 text-amber-700 text-xs disabled:opacity-50"
                >
                  {actionLoading === "reset" ? "重置中..." : "重置到基线"}
                </button>
                <button
                  type="button"
                  data-testid="employee-skill-os-archive"
                  disabled={!canArchive || Boolean(actionLoading)}
                  onClick={() => onRequestArchive(selectedEntry.skill_id)}
                  className="h-8 px-3 rounded border border-slate-200 bg-white hover:bg-slate-50 text-slate-700 text-xs disabled:opacity-50"
                >
                  {actionLoading === "archive" ? "归档中..." : "归档"}
                </button>
                {isArchived && (
                  <button
                    type="button"
                    data-testid="employee-skill-os-restore"
                    disabled={!canRestore || Boolean(actionLoading)}
                    onClick={() => onRequestRestore(selectedEntry.skill_id)}
                    className="h-8 px-3 rounded border border-emerald-200 bg-emerald-50 hover:bg-emerald-100 text-emerald-700 text-xs disabled:opacity-50"
                  >
                    {actionLoading === "restore" ? "恢复中..." : "恢复"}
                  </button>
                )}
                <button
                  type="button"
                  data-testid="employee-skill-os-delete"
                  disabled={!canDelete || Boolean(actionLoading)}
                  onClick={() => onRequestDelete(selectedEntry.skill_id)}
                  className="h-8 px-3 rounded border border-red-200 bg-red-50 hover:bg-red-100 text-red-700 text-xs disabled:opacity-50"
                >
                  {actionLoading === "delete" ? "删除中..." : "删除"}
                </button>
                {selectedEntry.source.immutable_content && (
                  <span className="text-[11px] text-gray-500">
                    .skillpack 受保护，不能被修改、归档、删除或重置。
                  </span>
                )}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                <div className="rounded border border-gray-100 p-2">
                  <div className="text-[11px] text-gray-500">生命周期能力</div>
                  <div data-testid="employee-skill-os-capability" className="text-gray-700">
                    {capabilityLabel(selectedEntry)}
                  </div>
                </div>
                <div className="rounded border border-gray-100 p-2">
                  <div className="text-[11px] text-gray-500">Toolsets</div>
                  <div data-testid="employee-skill-os-toolsets" className="text-gray-700">
                    {renderToolsets(selectedEntry)}
                  </div>
                </div>
                <div className="rounded border border-gray-100 p-2 md:col-span-2">
                  <div className="text-[11px] text-gray-500">Usage</div>
                  <div data-testid="employee-skill-os-usage" className="text-gray-700">
                    view {selectedEntry.usage.view_count} · use {selectedEntry.usage.use_count} · patch{" "}
                    {selectedEntry.usage.patch_count}
                  </div>
                </div>
              </div>

              <div>
                <div className="text-[11px] text-gray-500 mb-1">版本历史</div>
                {detailLoading ? (
                  <div className="text-xs text-gray-500">正在加载版本...</div>
                ) : selectedVersions.length === 0 ? (
                  <div className="rounded border border-dashed border-gray-200 p-2 text-xs text-gray-500">
                    暂无版本记录。
                  </div>
                ) : (
                  <div className="space-y-1" data-testid="employee-skill-os-versions">
                    {selectedVersions.map((version) => (
                      <div key={version.version_id} className="rounded border border-gray-100 p-2 text-xs">
                        <div className="flex items-center justify-between gap-2">
                          <span className="font-medium text-gray-800">{versionActionLabel(version.action)}</span>
                          <span className="text-[10px] text-gray-400">{version.created_at}</span>
                        </div>
                        <div className="mt-1 flex items-center justify-between gap-2">
                          <div className="text-[11px] text-gray-600 truncate">
                            {version.summary || version.version_id}
                          </div>
                          <button
                            type="button"
                            data-testid={`employee-skill-os-rollback-${version.version_id}`}
                            disabled={!selectedEntry.capabilities.can_patch || Boolean(actionLoading)}
                            onClick={() => onRequestRollback(version)}
                            className="h-6 shrink-0 rounded border border-blue-200 px-2 text-[10px] text-blue-700 hover:bg-blue-50 disabled:opacity-50"
                          >
                            {actionLoading === "rollback" ? "回滚中..." : "回滚"}
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {selectedView?.content && !editing && (
                <pre
                  data-testid="employee-skill-os-content"
                  className="max-h-32 overflow-auto rounded border border-gray-100 bg-gray-50 p-2 text-[11px] text-gray-700 whitespace-pre-wrap"
                >
                  {selectedView.content}
                </pre>
              )}

              {editing && (
                <div className="space-y-2" data-testid="employee-skill-os-editor">
                  <textarea
                    value={draftContent}
                    onChange={(event) => setDraftContent(event.target.value)}
                    className="h-48 w-full resize-y rounded border border-gray-200 bg-white p-2 font-mono text-[11px] text-gray-800 outline-none focus:border-blue-300"
                    spellCheck={false}
                  />
                  <div className="flex items-center justify-between gap-2">
                    <div className="text-[11px] text-gray-500">
                      {diff ? "将写入新的 Skill OS 版本和成长记录。" : "内容未变化。"}
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        type="button"
                        disabled={Boolean(actionLoading)}
                        onClick={() => {
                          setDraftContent(sourceContent);
                          setEditing(false);
                        }}
                        className="h-8 px-3 rounded border border-gray-200 hover:bg-gray-50 text-gray-700 text-xs disabled:opacity-50"
                      >
                        取消
                      </button>
                      <button
                        type="button"
                        data-testid="employee-skill-os-save-patch"
                        disabled={!diff || Boolean(actionLoading)}
                        onClick={() => selectedEntry && onRequestPatch(selectedEntry.skill_id, draftContent)}
                        className="h-8 px-3 rounded bg-blue-600 hover:bg-blue-700 text-white text-xs disabled:opacity-50"
                      >
                        {actionLoading === "patch" ? "保存中..." : "保存变更"}
                      </button>
                    </div>
                  </div>
                  {diff && (
                    <pre
                      data-testid="employee-skill-os-diff"
                      className="max-h-40 overflow-auto rounded border border-gray-100 bg-gray-50 p-2 text-[11px] text-gray-700 whitespace-pre-wrap"
                    >
                      {diff}
                    </pre>
                  )}
                </div>
              )}
            </>
          ) : (
            <div className="rounded border border-dashed border-gray-200 p-2 text-xs text-gray-500">
              该技能尚未进入 Skill OS index，保留为员工绑定标签。
            </div>
          )}
        </div>
      )}
    </div>
  );
}
