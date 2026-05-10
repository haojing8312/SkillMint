import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import {
  AgentEmployee,
  AgentProfileFilesView,
  EmployeeCuratorReports,
  EmployeeCuratorSchedulerStatus,
  EmployeeGrowthTimeline,
  EmployeeProfileMemoryStatus,
  SkillOsIndexEntry,
  SkillOsVersionEntry,
  SkillOsView,
} from "../../../types";
import {
  archiveSkillOs,
  deleteSkillOs,
  exportAgentProfile,
  getEmployeeCuratorReports,
  getEmployeeCuratorSchedulerStatus,
  getEmployeeGrowthTimeline,
  getEmployeeProfileMemoryStatus,
  getSkillOsView,
  listSkillOsIndex,
  listSkillOsVersions,
  patchSkillOs,
  pinSkillOs,
  resetSkillOs,
  restoreEmployeeCuratorStaleSkill,
  restoreSkillOs,
  rollbackSkillOs,
  scanEmployeeCuratorProfile,
} from "../../../scenes/employees/employeeHubApi";

export interface UseEmployeeHubToolsArgs {
  selectedEmployee: AgentEmployee | null;
  selectedEmployeeId: string | null;
  selectedEmployeeMemoryId: string | null;
  setMessage: (message: string) => void;
}

export function useEmployeeHubTools({
  selectedEmployee,
  selectedEmployeeId,
  selectedEmployeeMemoryId,
  setMessage,
}: UseEmployeeHubToolsArgs) {
  const [profileView, setProfileView] = useState<AgentProfileFilesView | null>(null);
  const [profileLoading, setProfileLoading] = useState(false);
  const [profileActionLoading, setProfileActionLoading] = useState<"export" | null>(null);
  const [profileMemoryStatus, setProfileMemoryStatus] =
    useState<EmployeeProfileMemoryStatus | null>(null);
  const [profileMemoryStatusLoading, setProfileMemoryStatusLoading] = useState(false);
  const [profileMemoryStatusError, setProfileMemoryStatusError] = useState("");
  const [growthTimeline, setGrowthTimeline] = useState<EmployeeGrowthTimeline | null>(null);
  const [growthTimelineLoading, setGrowthTimelineLoading] = useState(false);
  const [growthTimelineError, setGrowthTimelineError] = useState("");
  const [curatorReports, setCuratorReports] = useState<EmployeeCuratorReports | null>(null);
  const [curatorReportsLoading, setCuratorReportsLoading] = useState(false);
  const [curatorReportsError, setCuratorReportsError] = useState("");
  const [curatorSchedulerStatus, setCuratorSchedulerStatus] =
    useState<EmployeeCuratorSchedulerStatus | null>(null);
  const [curatorSchedulerError, setCuratorSchedulerError] = useState("");
  const [curatorActionLoading, setCuratorActionLoading] = useState<string | null>(null);
  const [skillOsIndex, setSkillOsIndex] = useState<SkillOsIndexEntry[]>([]);
  const [skillOsLoading, setSkillOsLoading] = useState(false);
  const [skillOsError, setSkillOsError] = useState("");
  const [selectedSkillOsId, setSelectedSkillOsId] = useState<string | null>(null);
  const [selectedSkillOsView, setSelectedSkillOsView] = useState<SkillOsView | null>(null);
  const [selectedSkillOsVersions, setSelectedSkillOsVersions] = useState<SkillOsVersionEntry[]>([]);
  const [selectedSkillOsDetailLoading, setSelectedSkillOsDetailLoading] = useState(false);
  const [skillOsActionLoading, setSkillOsActionLoading] =
    useState<"patch" | "pin" | "reset" | "rollback" | "archive" | "restore" | "delete" | null>(null);

  function safeFilePart(value: string): string {
    const normalized = value
      .trim()
      .replace(/[<>:"/\\|?*\x00-\x1F]/g, "-")
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-")
      .slice(0, 48);
    return normalized || "employee";
  }

  function dateStamp(): string {
    const date = new Date();
    const pad = (value: number) => String(value).padStart(2, "0");
    return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}`;
  }

  useEffect(() => {
    setProfileMemoryStatus(null);
    setProfileMemoryStatusError("");
    setProfileActionLoading(null);
    setGrowthTimeline(null);
    setGrowthTimelineError("");
    setCuratorReports(null);
    setCuratorReportsError("");
    setCuratorSchedulerStatus(null);
    setCuratorSchedulerError("");
    setCuratorActionLoading(null);
    setSkillOsIndex([]);
    setSkillOsError("");
    setSelectedSkillOsId(null);
    setSelectedSkillOsView(null);
    setSelectedSkillOsVersions([]);
  }, [selectedEmployeeId]);

  useEffect(() => {
    if (!selectedEmployee) {
      setProfileView(null);
      setProfileLoading(false);
      return;
    }

    let disposed = false;
    setProfileLoading(true);
    invoke<AgentProfileFilesView>("get_agent_profile_files", { employeeDbId: selectedEmployee.id })
      .then((view) => {
        if (!disposed) setProfileView(view);
      })
      .catch(() => {
        if (!disposed) setProfileView(null);
      })
      .finally(() => {
        if (!disposed) setProfileLoading(false);
      });

    return () => {
      disposed = true;
    };
  }, [selectedEmployee]);

  function resolveProfileMemoryStatusSkillId(scopeSkillId?: string) {
    const primary = selectedEmployee?.primary_skill_id?.trim();
    if (primary) {
      return primary;
    }
    return selectedEmployee?.skill_ids?.find((id) => id.trim())?.trim() || "builtin-general";
  }

  async function refreshProfileMemoryStatus(scopeSkillId?: string) {
    if (!selectedEmployeeMemoryId || !selectedEmployee) {
      setProfileMemoryStatus(null);
      setProfileMemoryStatusError("");
      return;
    }
    const skillId = resolveProfileMemoryStatusSkillId(scopeSkillId);
    setProfileMemoryStatusLoading(true);
    setProfileMemoryStatusError("");
    try {
      const status = await getEmployeeProfileMemoryStatus({
        employeeId: selectedEmployeeMemoryId,
        skillId,
        workDir: selectedEmployee.default_work_dir?.trim() || null,
      });
      setProfileMemoryStatus(status);
    } catch (e) {
      setProfileMemoryStatus(null);
      setProfileMemoryStatusError(String(e));
    } finally {
      setProfileMemoryStatusLoading(false);
    }
  }

  async function refreshGrowthTimeline() {
    if (!selectedEmployee) {
      setGrowthTimeline(null);
      setGrowthTimelineError("");
      return;
    }
    setGrowthTimelineLoading(true);
    setGrowthTimelineError("");
    try {
      const timeline = await getEmployeeGrowthTimeline({
        employeeId: selectedEmployee.id,
        limit: 12,
      });
      setGrowthTimeline(timeline);
    } catch (e) {
      setGrowthTimeline(null);
      setGrowthTimelineError(String(e));
    } finally {
      setGrowthTimelineLoading(false);
    }
  }

  async function exportSelectedAgentProfile() {
    if (!selectedEmployee || profileActionLoading) return;
    setProfileActionLoading("export");
    try {
      const filePath = await saveDialog({
        defaultPath: `workclaw-profile-${safeFilePart(selectedEmployee.name)}-${safeFilePart(
          selectedEmployee.employee_id || selectedEmployee.role_id || selectedEmployee.id,
        )}-${dateStamp()}.zip`,
        filters: [{ name: "ZIP", extensions: ["zip"] }],
      });
      if (!filePath) return;
      const result = await exportAgentProfile({
        employeeDbId: selectedEmployee.id,
        outputPath: filePath,
      });
      setMessage(`Profile 已导出：${result.file_count} 个文件，路径 ${result.export_path}`);
    } catch (e) {
      setMessage(`导出 Profile 失败: ${String(e)}`);
    } finally {
      setProfileActionLoading(null);
    }
  }

  async function refreshCuratorReports() {
    if (!selectedEmployee) {
      setCuratorReports(null);
      setCuratorReportsError("");
      return;
    }
    setCuratorReportsLoading(true);
    setCuratorReportsError("");
    try {
      const reports = await getEmployeeCuratorReports({
        employeeId: selectedEmployee.id,
        limit: 5,
      });
      setCuratorReports(reports);
    } catch (e) {
      setCuratorReports(null);
      setCuratorReportsError(String(e));
    } finally {
      setCuratorReportsLoading(false);
    }
  }

  async function refreshCuratorSchedulerStatus() {
    if (!selectedEmployee) {
      setCuratorSchedulerStatus(null);
      setCuratorSchedulerError("");
      return;
    }
    try {
      const status = await getEmployeeCuratorSchedulerStatus({
        employeeId: selectedEmployee.id,
      });
      setCuratorSchedulerStatus(status);
      setCuratorSchedulerError("");
    } catch (e) {
      setCuratorSchedulerStatus(null);
      setCuratorSchedulerError(String(e));
    }
  }

  async function restoreCuratorStaleSkill(skillId: string) {
    const normalized = skillId.trim();
    if (!selectedEmployee || !normalized || curatorActionLoading) return;
    setCuratorActionLoading(normalized);
    try {
      await restoreEmployeeCuratorStaleSkill({
        employeeId: selectedEmployee.id,
        skillId: normalized,
      });
      await Promise.all([
        refreshCuratorReports(),
        refreshCuratorSchedulerStatus(),
        refreshGrowthTimeline(),
        refreshSkillOsIndex(),
        selectedSkillOsId === normalized
          ? refreshSelectedSkillOsDetail(normalized)
          : Promise.resolve(),
      ]);
      setMessage("Curator 已恢复 stale skill");
    } catch (e) {
      setMessage(`恢复 Curator stale skill 失败: ${String(e)}`);
    } finally {
      setCuratorActionLoading(null);
    }
  }

  async function scanCuratorProfile(mode: "scan" | "run" = "scan") {
    if (!selectedEmployee || curatorActionLoading) return;
    setCuratorActionLoading(mode);
    try {
      await scanEmployeeCuratorProfile({
        employeeId: selectedEmployee.id,
        mode,
      });
      await Promise.all([
        refreshCuratorReports(),
        refreshCuratorSchedulerStatus(),
        refreshGrowthTimeline(),
        refreshSkillOsIndex(),
        selectedSkillOsId ? refreshSelectedSkillOsDetail(selectedSkillOsId) : Promise.resolve(),
      ]);
      setMessage(mode === "run" ? "Curator 已执行整理" : "Curator 扫描完成");
    } catch (e) {
      setMessage(`Curator ${mode === "run" ? "整理" : "扫描"}失败: ${String(e)}`);
    } finally {
      setCuratorActionLoading(null);
    }
  }

  async function refreshSkillOsIndex() {
    if (!selectedEmployee) {
      setSkillOsIndex([]);
      setSkillOsError("");
      return;
    }
    setSkillOsLoading(true);
    setSkillOsError("");
    try {
      const index = await listSkillOsIndex();
      setSkillOsIndex(index);
    } catch (e) {
      setSkillOsIndex([]);
      setSkillOsError(String(e));
    } finally {
      setSkillOsLoading(false);
    }
  }

  async function refreshSelectedSkillOsDetail(skillId?: string | null) {
    const normalized = (skillId ?? selectedSkillOsId ?? "").trim();
    if (!normalized) {
      setSelectedSkillOsView(null);
      setSelectedSkillOsVersions([]);
      return;
    }
    setSelectedSkillOsDetailLoading(true);
    setSkillOsError("");
    try {
      const [view, versions] = await Promise.all([
        getSkillOsView(normalized),
        listSkillOsVersions({ skillId: normalized, limit: 8 }),
      ]);
      setSelectedSkillOsView(view);
      setSelectedSkillOsVersions(versions);
    } catch (e) {
      setSelectedSkillOsView(null);
      setSelectedSkillOsVersions([]);
      setSkillOsError(String(e));
    } finally {
      setSelectedSkillOsDetailLoading(false);
    }
  }

  function selectSkillOs(skillId: string) {
    const normalized = skillId.trim();
    setSelectedSkillOsId(normalized || null);
  }

  async function resetSelectedSkillOs() {
    const skillId = (selectedSkillOsId ?? "").trim();
    if (!skillId || skillOsActionLoading) return;
    setSkillOsActionLoading("reset");
    try {
      await resetSkillOs({
        skillId,
        employeeId: selectedEmployee?.id ?? null,
        summary: "Reset skill from employee workbench",
        confirm: true,
      });
      await refreshSelectedSkillOsDetail(skillId);
      await refreshSkillOsIndex();
      await refreshGrowthTimeline();
      setMessage("技能已重置到基线版本");
    } catch (e) {
      setMessage(`重置技能失败: ${String(e)}`);
    } finally {
      setSkillOsActionLoading(null);
    }
  }

  async function patchSelectedSkillOs(content: string) {
    const skillId = (selectedSkillOsId ?? "").trim();
    if (!skillId || skillOsActionLoading) return;
    setSkillOsActionLoading("patch");
    try {
      await patchSkillOs({
        skillId,
        content,
        employeeId: selectedEmployee?.id ?? null,
        summary: "Patch skill from employee workbench",
        confirm: true,
      });
      await refreshSelectedSkillOsDetail(skillId);
      await refreshSkillOsIndex();
      await refreshGrowthTimeline();
      setMessage("技能已更新");
    } catch (e) {
      setMessage(`更新技能失败: ${String(e)}`);
    } finally {
      setSkillOsActionLoading(null);
    }
  }

  async function setSelectedSkillOsPinned(pinned: boolean) {
    const skillId = (selectedSkillOsId ?? "").trim();
    if (!skillId || skillOsActionLoading) return;
    setSkillOsActionLoading("pin");
    try {
      await pinSkillOs({ skillId, pinned });
      await refreshSelectedSkillOsDetail(skillId);
      await refreshSkillOsIndex();
      setMessage(pinned ? "技能已固定，Curator 会跳过自动整理" : "技能已取消固定");
    } catch (e) {
      setMessage(`更新技能固定状态失败: ${String(e)}`);
    } finally {
      setSkillOsActionLoading(null);
    }
  }

  async function rollbackSelectedSkillOs(versionId: string) {
    const skillId = (selectedSkillOsId ?? "").trim();
    const normalizedVersionId = versionId.trim();
    if (!skillId || !normalizedVersionId || skillOsActionLoading) return;
    setSkillOsActionLoading("rollback");
    try {
      await rollbackSkillOs({
        skillId,
        versionId: normalizedVersionId,
        employeeId: selectedEmployee?.id ?? null,
        summary: "Rollback skill from employee workbench",
        confirm: true,
      });
      await refreshSelectedSkillOsDetail(skillId);
      await refreshSkillOsIndex();
      await refreshGrowthTimeline();
      setMessage("技能已回滚到选定版本");
    } catch (e) {
      setMessage(`回滚技能失败: ${String(e)}`);
    } finally {
      setSkillOsActionLoading(null);
    }
  }

  async function archiveSelectedSkillOs() {
    const skillId = (selectedSkillOsId ?? "").trim();
    if (!skillId || skillOsActionLoading) return;
    setSkillOsActionLoading("archive");
    try {
      await archiveSkillOs({
        skillId,
        employeeId: selectedEmployee?.id ?? null,
        summary: "Archive skill from employee workbench",
        confirm: true,
      });
      await refreshSelectedSkillOsDetail(skillId);
      await refreshSkillOsIndex();
      await refreshGrowthTimeline();
      setMessage("技能已归档");
    } catch (e) {
      setMessage(`归档技能失败: ${String(e)}`);
    } finally {
      setSkillOsActionLoading(null);
    }
  }

  async function restoreSelectedSkillOs() {
    const skillId = (selectedSkillOsId ?? "").trim();
    if (!skillId || skillOsActionLoading) return;
    setSkillOsActionLoading("restore");
    try {
      await restoreSkillOs({
        skillId,
        employeeId: selectedEmployee?.id ?? null,
        summary: "Restore skill from employee workbench",
      });
      await refreshSelectedSkillOsDetail(skillId);
      await refreshSkillOsIndex();
      await refreshGrowthTimeline();
      setMessage("技能已恢复");
    } catch (e) {
      setMessage(`恢复技能失败: ${String(e)}`);
    } finally {
      setSkillOsActionLoading(null);
    }
  }

  async function deleteSelectedSkillOs() {
    const skillId = (selectedSkillOsId ?? "").trim();
    if (!skillId || skillOsActionLoading) return;
    setSkillOsActionLoading("delete");
    try {
      await deleteSkillOs({
        skillId,
        employeeId: selectedEmployee?.id ?? null,
        summary: "Delete skill from employee workbench",
        confirm: true,
      });
      await refreshSkillOsIndex();
      await refreshGrowthTimeline();
      await refreshSelectedSkillOsDetail(skillId);
      setMessage("技能已删除");
    } catch (e) {
      setMessage(`删除技能失败: ${String(e)}`);
    } finally {
      setSkillOsActionLoading(null);
    }
  }

  useEffect(() => {
    if (!selectedEmployeeMemoryId) {
      setProfileMemoryStatus(null);
      setGrowthTimeline(null);
      setCuratorReports(null);
      setCuratorSchedulerStatus(null);
      setSkillOsIndex([]);
      setSelectedSkillOsId(null);
      setSelectedSkillOsView(null);
      setSelectedSkillOsVersions([]);
      return;
    }
    void refreshProfileMemoryStatus();
    void refreshGrowthTimeline();
    void refreshCuratorReports();
    void refreshCuratorSchedulerStatus();
    void refreshSkillOsIndex();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedEmployeeMemoryId, selectedEmployee?.id, selectedEmployee?.primary_skill_id]);

  useEffect(() => {
    if (!selectedEmployee) {
      setSelectedSkillOsId(null);
      return;
    }
    const candidates = [
      selectedEmployee.primary_skill_id,
      ...(selectedEmployee.skill_ids ?? []),
    ].map((id) => id.trim()).filter(Boolean);
    setSelectedSkillOsId((current) => {
      if (current && candidates.includes(current)) return current;
      return candidates[0] ?? null;
    });
  }, [selectedEmployee]);

  useEffect(() => {
    void refreshSelectedSkillOsDetail(selectedSkillOsId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedSkillOsId]);

  return {
    profileView,
    profileLoading,
    profileActionLoading,
    profileMemoryStatus,
    profileMemoryStatusLoading,
    profileMemoryStatusError,
    growthTimeline,
    growthTimelineLoading,
    growthTimelineError,
    curatorReports,
    curatorReportsLoading,
    curatorReportsError,
    curatorSchedulerStatus,
    curatorSchedulerError,
    curatorActionLoading,
    skillOsIndex,
    skillOsLoading,
    skillOsError,
    selectedSkillOsId,
    selectedSkillOsView,
    selectedSkillOsVersions,
    selectedSkillOsDetailLoading,
    skillOsActionLoading,
    refreshProfileMemoryStatus,
    refreshGrowthTimeline,
    exportSelectedAgentProfile,
    refreshCuratorReports,
    refreshCuratorSchedulerStatus,
    scanCuratorProfile,
    restoreCuratorStaleSkill,
    refreshSkillOsIndex,
    refreshSelectedSkillOsDetail,
    selectSkillOs,
    patchSelectedSkillOs,
    setSelectedSkillOsPinned,
    resetSelectedSkillOs,
    rollbackSelectedSkillOs,
    archiveSelectedSkillOs,
    restoreSelectedSkillOs,
    deleteSelectedSkillOs,
  };
}
