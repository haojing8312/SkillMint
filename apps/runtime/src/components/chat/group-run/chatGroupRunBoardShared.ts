import type { MutableRefObject } from "react";

import type { EmployeeGroupRunSnapshot } from "../../../types";

export type GroupRunOpenSessionOptions = {
  focusHint?: string;
  sourceSessionId?: string;
  sourceStepId?: string;
  sourceEmployeeId?: string;
  assigneeEmployeeId?: string;
  sourceStepTimeline?: Array<{
    eventId?: string;
    linkedSessionId?: string;
    label: string;
    createdAt?: string;
  }>;
  groupRunStepFocusId?: string;
  groupRunEventFocusId?: string;
};

export type GroupRunStepTimelineItem = {
  eventId?: string;
  linkedSessionId?: string;
  label: string;
  createdAt?: string;
  openSessionOptions?: GroupRunOpenSessionOptions;
};

export type GroupRunExecuteStepCard = {
  step: EmployeeGroupRunSnapshot["steps"][number];
  currentAssigneeEmployeeId: string;
  dispatchSourceEmployeeId: string;
  previousAssigneeEmployeeId: string;
  latestFailureSummary: string;
  attemptNo: number;
  detailSessionId: string;
  detailOutputSummary: string;
  latestEventCreatedAt: string;
  sourceStepTimeline: GroupRunStepTimelineItem[];
  detailSessionOpenOptions?: GroupRunOpenSessionOptions;
};

export type GroupRunReassignOption = {
  step: EmployeeGroupRunSnapshot["steps"][number];
  candidateEmployeeIds: string[];
};

export type GroupRunActionLoading = "approve" | "reject" | "pause" | "resume" | "retry" | "reassign" | null;

export type GroupRunMemberState = {
  role: string;
  status: string;
  stepType?: string;
};

export type GroupRunRecentEvent = {
  id: string;
  label: string;
};

export type ChatGroupRunBoardProps = {
  groupPhaseLabel: string | null;
  groupRound: number;
  groupReviewRound: number;
  groupWaitingLabel: string;
  groupStatusReason: string;
  groupRunSnapshot: EmployeeGroupRunSnapshot | null;
  onApproveGroupRunReview: () => void;
  onRejectGroupRunReview: () => void;
  onPauseGroupRun: () => void;
  onResumeGroupRun: () => void;
  onRetryFailedGroupRunSteps: () => void;
  onReassignFailedGroupRunStep: (stepId: string, employeeId: string) => void;
  groupRunActionLoading: GroupRunActionLoading;
  canPauseGroupRun: boolean;
  canResumeGroupRun: boolean;
  canRetryFailedGroupRunSteps: boolean;
  canReassignFailedGroupRunStep: boolean;
  failedGroupRunReassignOptions: GroupRunReassignOption[];
  groupMemberStates: GroupRunMemberState[];
  recentGroupEvents: GroupRunRecentEvent[];
  groupRunExecuteStepCards: GroupRunExecuteStepCard[];
  highlightedGroupRunStepId: string | null;
  highlightedGroupRunStepEventId: string | null;
  expandedGroupRunStepIds: string[];
  groupRunStepElementRefs: MutableRefObject<Record<string, HTMLDivElement | null>>;
  groupRunStepEventElementRefs: MutableRefObject<Record<string, HTMLDivElement | HTMLButtonElement | null>>;
  onToggleGroupRunStepDetails: (stepId: string) => void;
  onOpenSession?: (sessionId: string, options?: GroupRunOpenSessionOptions) => Promise<void> | void;
  sessionId: string;
};

export function formatGroupRunStepStatusLabel(status?: string) {
  const normalized = (status || "").trim().toLowerCase();
  if (normalized === "completed" || normalized === "done") return "已完成";
  if (normalized === "failed") return "失败";
  if (normalized === "running" || normalized === "executing") return "执行中";
  if (normalized === "pending") return "待执行";
  if (normalized === "paused") return "已暂停";
  if (normalized === "cancelled") return "已取消";
  return status?.trim() || "待执行";
}
