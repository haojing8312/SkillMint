import type {
  AgentProfileFilesView,
  EmployeeCuratorReports,
  EmployeeGrowthTimeline,
  EmployeeProfileMemoryStatus,
  SkillOsIndexEntry,
} from "../../../types";

interface EmployeeProfileRuntimeStatusPanelProps {
  authorizedSkillIds: string[];
  curatorReports: EmployeeCuratorReports | null;
  growthTimeline: EmployeeGrowthTimeline | null;
  profileMemoryStatus: EmployeeProfileMemoryStatus | null;
  profileView: AgentProfileFilesView | null;
  skillOsIndex: SkillOsIndexEntry[];
}

function resolveProfileId({
  profileMemoryStatus,
  growthTimeline,
  curatorReports,
  profileView,
}: Pick<
  EmployeeProfileRuntimeStatusPanelProps,
  "profileMemoryStatus" | "growthTimeline" | "curatorReports" | "profileView"
>): string {
  const candidates = [
    profileMemoryStatus?.profile_id,
    growthTimeline?.profile_id,
    curatorReports?.profile_id,
    profileView?.profile_dir?.split(/[\\/]/).filter(Boolean).pop(),
  ];
  return candidates.find((item) => item && item.trim())?.trim() || "未解析";
}

function statusTone(ok: boolean): string {
  return ok
    ? "border-emerald-100 bg-emerald-50 text-emerald-800"
    : "border-amber-100 bg-amber-50 text-amber-800";
}

function uniqueRuntimeToolsets(authorizedSkillIds: string[], skillOsIndex: SkillOsIndexEntry[]): string[] {
  const authorized = new Set(authorizedSkillIds.map((item) => item.trim()).filter(Boolean));
  const entries = skillOsIndex.filter((entry) => authorized.size === 0 || authorized.has(entry.skill_id));
  const ordered = new Set<string>();
  for (const entry of entries) {
    for (const toolset of [
      ...entry.toolset_policy.requires_toolsets,
      ...entry.toolset_policy.optional_toolsets,
    ]) {
      const normalized = toolset.trim();
      if (!normalized || normalized === "skills") continue;
      ordered.add(normalized);
    }
  }
  return Array.from(ordered.values());
}

export function EmployeeProfileRuntimeStatusPanel({
  authorizedSkillIds,
  curatorReports,
  growthTimeline,
  profileMemoryStatus,
  profileView,
  skillOsIndex,
}: EmployeeProfileRuntimeStatusPanelProps) {
  const profileId = resolveProfileId({ profileMemoryStatus, growthTimeline, curatorReports, profileView });
  const profileHomeReady = Boolean(profileView?.profile_dir?.trim());
  const memoryReady = profileMemoryStatus?.active_source === "profile" && profileMemoryStatus.profile_memory_file_exists;
  const skillCount = authorizedSkillIds.length;
  const toolsets = uniqueRuntimeToolsets(authorizedSkillIds, skillOsIndex);
  const growthCount = growthTimeline?.events.length ?? 0;
  const curatorCount = curatorReports?.runs.length ?? 0;

  return (
    <div
      data-testid="employee-profile-runtime-status"
      className="rounded-lg border border-indigo-100 bg-indigo-50/70 p-3 space-y-3"
    >
      <div className="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="text-sm font-semibold text-indigo-950">AI 员工运行时状态</div>
          <div className="mt-1 text-[11px] text-indigo-800">
            Canonical Profile Runtime · profile_id → profiles/&lt;profile_id&gt;/...
          </div>
        </div>
        <div
          className="max-w-[260px] truncate rounded border border-indigo-200 bg-white px-2 py-1 text-[10px] text-indigo-700"
          title={profileId}
        >
          {profileId}
        </div>
      </div>

      <div className="grid grid-cols-1 gap-2 md:grid-cols-3">
        <div className={`rounded border px-2 py-1.5 ${statusTone(profileHomeReady)}`}>
          <div className="text-[10px] uppercase opacity-70">Profile Home</div>
          <div className="text-[11px] font-medium">{profileHomeReady ? "目录已就绪" : "等待创建"}</div>
        </div>
        <div data-testid="employee-profile-runtime-memory" className={`rounded border px-2 py-1.5 ${statusTone(memoryReady)}`}>
          <div className="text-[10px] uppercase opacity-70">Memory OS</div>
          <div className="text-[11px] font-medium">{memoryReady ? "Profile Memory 可用" : "暂无 Profile Memory"}</div>
        </div>
        <div data-testid="employee-profile-runtime-skills" className={`rounded border px-2 py-1.5 ${statusTone(skillCount > 0)}`}>
          <div className="text-[10px] uppercase opacity-70">Skill OS</div>
          <div className="text-[11px] font-medium">
            {skillCount > 0 ? `${skillCount} 个技能授权` : "默认通用技能"}
          </div>
        </div>
        <div data-testid="employee-profile-runtime-toolsets" className={`rounded border px-2 py-1.5 ${statusTone(toolsets.length > 0)}`}>
          <div className="text-[10px] uppercase opacity-70">Toolsets</div>
          <div className="text-[11px] font-medium">{toolsets.length > 0 ? toolsets.join(" · ") : "未声明"}</div>
        </div>
        <div data-testid="employee-profile-runtime-growth" className={`rounded border px-2 py-1.5 ${statusTone(growthCount > 0)}`}>
          <div className="text-[10px] uppercase opacity-70">Growth</div>
          <div className="text-[11px] font-medium">
            {growthCount > 0 ? `${growthCount} 条成长证据` : "暂无成长证据"}
          </div>
        </div>
        <div data-testid="employee-profile-runtime-curator" className={`rounded border px-2 py-1.5 ${statusTone(curatorCount > 0)}`}>
          <div className="text-[10px] uppercase opacity-70">Curator</div>
          <div className="text-[11px] font-medium">
            {curatorCount > 0 ? `${curatorCount} 份 Curator 报告` : "暂无 Curator 报告"}
          </div>
        </div>
      </div>
    </div>
  );
}
