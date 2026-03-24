import { invoke } from "@tauri-apps/api/core";

import type { EmployeeGroup, EmployeeGroupRule, EmployeeGroupRunSnapshot } from "../../types";

export async function getEmployeeGroupRunSnapshot(sessionId: string): Promise<EmployeeGroupRunSnapshot | null> {
  return await invoke<EmployeeGroupRunSnapshot | null>("get_employee_group_run_snapshot", { sessionId });
}

export async function listEmployeeGroups(): Promise<EmployeeGroup[]> {
  const groups = await invoke<EmployeeGroup[] | null>("list_employee_groups");
  return Array.isArray(groups) ? groups : [];
}

export async function listEmployeeGroupRules(groupId: string): Promise<EmployeeGroupRule[]> {
  const rules = await invoke<EmployeeGroupRule[] | null>("list_employee_group_rules", { groupId });
  return Array.isArray(rules) ? rules : [];
}

export async function reviewGroupRunStep(
  runId: string,
  action: "approve" | "reject",
  comment: string,
): Promise<void> {
  await invoke("review_group_run_step", {
    runId,
    action,
    comment,
  });
}

export async function continueEmployeeGroupRun(runId: string): Promise<EmployeeGroupRunSnapshot> {
  return await invoke<EmployeeGroupRunSnapshot>("continue_employee_group_run", {
    runId,
  });
}

export async function pauseEmployeeGroupRun(runId: string, reason: string): Promise<void> {
  await invoke("pause_employee_group_run", {
    runId,
    reason,
  });
}

export async function resumeEmployeeGroupRun(runId: string): Promise<void> {
  await invoke("resume_employee_group_run", {
    runId,
  });
}

export async function retryEmployeeGroupRunFailedSteps(runId: string): Promise<void> {
  await invoke("retry_employee_group_run_failed_steps", {
    runId,
  });
}

export async function reassignGroupRunStep(stepId: string, assigneeEmployeeId: string): Promise<void> {
  await invoke("reassign_group_run_step", {
    stepId,
    assigneeEmployeeId,
  });
}
