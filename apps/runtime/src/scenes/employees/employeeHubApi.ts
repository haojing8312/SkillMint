import { invoke } from "@tauri-apps/api/core";
import {
  type AgentEmployee,
  type EmployeeCuratorReports,
  type EmployeeCuratorRun,
  type EmployeeCuratorSchedulerStatus,
  type EmployeeGroup,
  type EmployeeGrowthTimeline,
  type EmployeeProfileMemoryStatus,
  type AgentProfileExportResult,
  type SkillOsIndexEntry,
  type SkillOsMutationResult,
  type SkillOsVersionEntry,
  type SkillOsView,
  type UpsertAgentEmployeeInput,
} from "../../types";

export async function listAgentEmployees(): Promise<AgentEmployee[]> {
  const raw = await invoke<AgentEmployee[] | null>("list_agent_employees");
  return Array.isArray(raw) ? raw : [];
}

export async function listEmployeeGroups(): Promise<EmployeeGroup[]> {
  const raw = await invoke<EmployeeGroup[] | null>("list_employee_groups");
  return Array.isArray(raw) ? raw : [];
}

export async function upsertAgentEmployee(input: UpsertAgentEmployeeInput): Promise<string> {
  return invoke<string>("upsert_agent_employee", { input });
}

export async function deleteAgentEmployee(employeeId: string): Promise<void> {
  await invoke("delete_agent_employee", { employeeId });
}

export async function exportAgentProfile(input: {
  employeeDbId: string;
  outputPath: string;
}): Promise<AgentProfileExportResult> {
  return invoke<AgentProfileExportResult>("export_agent_profile", {
    employeeDbId: input.employeeDbId,
    outputPath: input.outputPath,
  });
}

export async function getEmployeeProfileMemoryStatus(input: {
  employeeId: string;
  skillId: string;
  profileId?: string | null;
  workDir?: string | null;
  imRoleId?: string | null;
}): Promise<EmployeeProfileMemoryStatus> {
  return invoke<EmployeeProfileMemoryStatus>("get_employee_profile_memory_status", {
    employeeId: input.employeeId,
    skillId: input.skillId,
    profileId: input.profileId ?? null,
    workDir: input.workDir ?? null,
    imRoleId: input.imRoleId ?? null,
  });
}

export async function getEmployeeGrowthTimeline(input: {
  employeeId: string;
  limit?: number;
}): Promise<EmployeeGrowthTimeline> {
  const raw = await invoke<EmployeeGrowthTimeline | null>("list_employee_growth_events", {
    employeeId: input.employeeId,
    limit: input.limit ?? 20,
  });
  return raw ?? { employee_id: input.employeeId, profile_id: null, events: [] };
}

export async function getEmployeeCuratorReports(input: {
  employeeId: string;
  limit?: number;
}): Promise<EmployeeCuratorReports> {
  const raw = await invoke<EmployeeCuratorReports | null>("list_employee_curator_runs", {
    employeeId: input.employeeId,
    limit: input.limit ?? 5,
  });
  return raw ?? { employee_id: input.employeeId, profile_id: null, runs: [] };
}

export async function scanEmployeeCuratorProfile(input: {
  employeeId: string;
  mode?: "scan" | "run";
}): Promise<EmployeeCuratorRun> {
  return invoke<EmployeeCuratorRun>("scan_employee_curator_profile", {
    employeeId: input.employeeId,
    mode: input.mode ?? "scan",
  });
}

export async function restoreEmployeeCuratorStaleSkill(input: {
  employeeId: string;
  skillId: string;
}): Promise<EmployeeCuratorRun> {
  return invoke<EmployeeCuratorRun>("restore_employee_curator_stale_skill", {
    employeeId: input.employeeId,
    skillId: input.skillId,
  });
}

export async function getEmployeeCuratorSchedulerStatus(input: {
  employeeId?: string | null;
}): Promise<EmployeeCuratorSchedulerStatus> {
  return invoke<EmployeeCuratorSchedulerStatus>("get_curator_scheduler_status", {
    employeeId: input.employeeId ?? null,
  });
}

export async function listSkillOsIndex(): Promise<SkillOsIndexEntry[]> {
  const raw = await invoke<SkillOsIndexEntry[] | null>("list_skill_os_index");
  return Array.isArray(raw) ? raw : [];
}

export async function getSkillOsView(skillId: string): Promise<SkillOsView | null> {
  return invoke<SkillOsView | null>("get_skill_os_view", { skillId });
}

export async function listSkillOsVersions(input: {
  skillId: string;
  limit?: number;
}): Promise<SkillOsVersionEntry[]> {
  const raw = await invoke<SkillOsVersionEntry[] | null>("list_skill_os_versions", {
    skillId: input.skillId,
    limit: input.limit ?? 8,
  });
  return Array.isArray(raw) ? raw : [];
}

export async function patchSkillOs(input: {
  skillId: string;
  content: string;
  employeeId?: string | null;
  summary?: string;
  confirm: boolean;
}): Promise<SkillOsMutationResult> {
  return invoke<SkillOsMutationResult>("patch_skill_os", {
    skillId: input.skillId,
    content: input.content,
    employeeId: input.employeeId ?? null,
    summary: input.summary ?? null,
    confirm: input.confirm,
  });
}

export async function rollbackSkillOs(input: {
  skillId: string;
  versionId: string;
  employeeId?: string | null;
  summary?: string;
  confirm: boolean;
}): Promise<SkillOsMutationResult> {
  return invoke<SkillOsMutationResult>("rollback_skill_os", {
    skillId: input.skillId,
    versionId: input.versionId,
    employeeId: input.employeeId ?? null,
    summary: input.summary ?? null,
    confirm: input.confirm,
  });
}

export async function resetSkillOs(input: {
  skillId: string;
  employeeId?: string | null;
  summary?: string;
  confirm: boolean;
}): Promise<SkillOsMutationResult> {
  return invoke<SkillOsMutationResult>("reset_skill_os", {
    skillId: input.skillId,
    employeeId: input.employeeId ?? null,
    summary: input.summary ?? null,
    confirm: input.confirm,
  });
}

export async function archiveSkillOs(input: {
  skillId: string;
  employeeId?: string | null;
  summary?: string;
  confirm: boolean;
}): Promise<SkillOsMutationResult> {
  return invoke<SkillOsMutationResult>("archive_skill_os", {
    skillId: input.skillId,
    employeeId: input.employeeId ?? null,
    summary: input.summary ?? null,
    confirm: input.confirm,
  });
}

export async function restoreSkillOs(input: {
  skillId: string;
  employeeId?: string | null;
  summary?: string;
}): Promise<SkillOsMutationResult> {
  return invoke<SkillOsMutationResult>("restore_skill_os", {
    skillId: input.skillId,
    employeeId: input.employeeId ?? null,
    summary: input.summary ?? null,
  });
}

export async function deleteSkillOs(input: {
  skillId: string;
  employeeId?: string | null;
  summary?: string;
  confirm: boolean;
}): Promise<SkillOsMutationResult> {
  return invoke<SkillOsMutationResult>("delete_skill_os", {
    skillId: input.skillId,
    employeeId: input.employeeId ?? null,
    summary: input.summary ?? null,
    confirm: input.confirm,
  });
}

export async function pinSkillOs(input: {
  skillId: string;
  pinned: boolean;
}): Promise<void> {
  await invoke("pin_skill_os", {
    skillId: input.skillId,
    pinned: input.pinned,
  });
}

export function buildDefaultEmployeeUpdateInput(
  employee: AgentEmployee,
): UpsertAgentEmployeeInput {
  return {
    id: employee.id,
    employee_id: employee.employee_id || employee.role_id,
    name: employee.name,
    role_id: employee.employee_id || employee.role_id,
    persona: employee.persona,
    feishu_open_id: employee.feishu_open_id,
    feishu_app_id: employee.feishu_app_id,
    feishu_app_secret: employee.feishu_app_secret,
    primary_skill_id: employee.primary_skill_id,
    default_work_dir: employee.default_work_dir,
    openclaw_agent_id:
      employee.employee_id || employee.openclaw_agent_id || employee.role_id,
    routing_priority: employee.routing_priority ?? 100,
    enabled_scopes: employee.enabled_scopes?.length
      ? employee.enabled_scopes
      : ["app"],
    enabled: employee.enabled,
    is_default: true,
    skill_ids: employee.skill_ids,
  };
}
