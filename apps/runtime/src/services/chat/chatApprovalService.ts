import { invoke } from "@tauri-apps/api/core";

export type PendingApprovalRecord = {
  approval_id: string;
  session_id: string;
  tool_name: string;
  input?: Record<string, unknown>;
  summary: string;
  impact?: string | null;
  irreversible?: boolean;
  status?: string;
};

export async function listPendingApprovals(sessionId: string): Promise<PendingApprovalRecord[]> {
  const approvals = await invoke<PendingApprovalRecord[] | null>("list_pending_approvals", {
    sessionId,
  });
  return Array.isArray(approvals) ? approvals : [];
}

export async function resolveApproval(
  approvalId: string,
  decision: "allow_once" | "allow_always" | "deny",
  source: "desktop" | "desktop_cleanup",
): Promise<void> {
  await invoke("resolve_approval", {
    approvalId,
    decision,
    source,
  });
}

export async function confirmLegacyToolExecution(confirmed: boolean): Promise<void> {
  await invoke("confirm_tool_execution", {
    confirmed,
  });
}
