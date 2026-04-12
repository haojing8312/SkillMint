import type {
  ChatDelegationCardState,
  EmployeeGroupRule,
  EmployeeGroupRunEvent,
  EmployeeGroupRunSnapshot,
} from "../../types";

export type GroupRunTimelineEntry = {
  eventId?: string;
  linkedSessionId?: string;
  label: string;
  createdAt?: string;
};

export type GroupRunTimelineItem = GroupRunTimelineEntry & {
  openSessionOptions?: {
    focusHint?: string;
    sourceSessionId?: string;
    sourceStepId?: string;
    sourceEmployeeId?: string;
    assigneeEmployeeId?: string;
    sourceStepTimeline?: GroupRunTimelineEntry[];
  };
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
  sourceStepTimeline: GroupRunTimelineItem[];
  detailSessionOpenOptions?: {
    focusHint?: string;
    sourceSessionId?: string;
    sourceStepId?: string;
    sourceEmployeeId?: string;
    assigneeEmployeeId?: string;
    sourceStepTimeline?: GroupRunTimelineEntry[];
  };
};

export type GroupRunReassignOption = {
  step: EmployeeGroupRunSnapshot["steps"][number];
  candidateEmployeeIds: string[];
};

export function deriveDelegationState(args: {
  delegationCards: ChatDelegationCardState[];
  mainRoleName: string;
  mainSummaryDelivered: boolean;
}) {
  const { delegationCards, mainRoleName, mainSummaryDelivered } = args;
  const activeDelegationCard = [...delegationCards].reverse().find((card) => card.status === "running");
  const primaryDelegationCard =
    activeDelegationCard || (delegationCards.length > 0 ? delegationCards[delegationCards.length - 1] : null);
  const delegationHistoryCards = primaryDelegationCard
    ? delegationCards.filter((card) => card.id !== primaryDelegationCard.id)
    : [];
  const runningDelegationCount = delegationCards.filter((card) => card.status === "running").length;
  const completedDelegationCount = delegationCards.filter((card) => card.status === "completed").length;
  const failedDelegationCount = delegationCards.filter((card) => card.status === "failed").length;
  const latestCompletedDelegation = [...delegationCards].reverse().find((card) => card.status === "completed");
  const groupPhaseFromEvents = mainSummaryDelivered
    ? "汇报"
    : delegationCards.length > 0
    ? "执行"
    : mainRoleName
    ? "计划"
    : null;
  const groupRoundFromEvents = delegationCards.length > 0 ? Math.max(1, Math.ceil(delegationCards.length / 3)) : 0;
  const groupMemberStatesFromEvents = (() => {
    const byRole = new Map<string, { status: "running" | "completed" | "failed"; stepType: string }>();
    for (const card of delegationCards) {
      byRole.set(card.toRole, { status: card.status, stepType: "execute" });
    }
    return Array.from(byRole.entries()).map(([role, info]) => ({
      role,
      status: info.status,
      stepType: info.stepType,
    }));
  })();
  const collaborationStatusText =
    mainSummaryDelivered
      ? `${mainRoleName || "主员工"} 已输出最终汇总`
      : runningDelegationCount > 0 && primaryDelegationCard
      ? `${mainRoleName || "主员工"} 正在处理，已委派 ${primaryDelegationCard.toRole}`
      : latestCompletedDelegation
      ? `${latestCompletedDelegation.toRole} 已完成，${mainRoleName || "主员工"} 正在汇总最终答复`
      : `${mainRoleName || "主员工"} 正在处理`;

  return {
    primaryDelegationCard,
    delegationHistoryCards,
    completedDelegationCount,
    failedDelegationCount,
    groupPhaseFromEvents,
    groupRoundFromEvents,
    groupMemberStatesFromEvents,
    collaborationStatusText,
  };
}

export function parseGroupRunEventPayload(event: EmployeeGroupRunEvent) {
  try {
    return event.payload_json ? (JSON.parse(event.payload_json) as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

function getGroupPhaseLabelFromSnapshot(snapshot: EmployeeGroupRunSnapshot | null) {
  const phase = (snapshot?.current_phase || "").trim().toLowerCase();
  const state = (snapshot?.state || "").trim().toLowerCase();
  if (state === "paused") return "已暂停";
  if (state === "failed") return "失败";
  if (state === "cancelled") return "已取消";
  const normalized = phase || state;
  if (!normalized) return null;
  if (normalized === "intake" || normalized === "plan" || normalized === "planning") return "计划";
  if (normalized === "review" || normalized === "waiting_review") return "审核";
  if (normalized === "dispatch" || normalized === "execute" || normalized === "executing") return "执行";
  if (normalized === "synthesize" || normalized === "finalize" || normalized === "done" || normalized === "completed") return "汇报";
  if (normalized === "failed") return "失败";
  if (normalized === "paused") return "已暂停";
  if (normalized === "cancelled") return "已取消";
  return "执行";
}

function getGroupPhaseFromSnapshot(snapshot: EmployeeGroupRunSnapshot | null) {
  const state = (snapshot?.state || "").trim().toLowerCase();
  if (!state) return null;
  if (state === "planning") return "计划";
  if (state === "executing") return "执行";
  if (state === "done" || state === "completed") return "汇报";
  if (state === "failed") return "失败";
  if (state === "cancelled") return "已取消";
  return "执行";
}

function formatGroupRunEventLabel(
  event: EmployeeGroupRunEvent,
  groupRunStepMap: Map<string, EmployeeGroupRunSnapshot["steps"][number]>,
) {
  const payload = parseGroupRunEventPayload(event);
  const relatedStep = groupRunStepMap.get(event.step_id);
  const assigneeEmployeeId = String(payload.assignee_employee_id || relatedStep?.assignee_employee_id || "").trim();
  const dispatchSourceEmployeeId = String(
    payload.dispatch_source_employee_id || relatedStep?.dispatch_source_employee_id || "",
  ).trim();
  if (["step_created", "step_dispatched", "step_completed", "step_failed", "step_reassigned"].includes(event.event_type)) {
    if (dispatchSourceEmployeeId && assigneeEmployeeId) {
      return `${event.event_type} · ${dispatchSourceEmployeeId} -> ${assigneeEmployeeId}`;
    }
    if (assigneeEmployeeId) {
      return `${event.event_type} · ${assigneeEmployeeId}`;
    }
  }
  return event.event_type;
}

function getGroupRunExecuteRuleTargets(args: {
  dispatchSourceEmployeeId?: string;
  groupRunCoordinatorEmployeeId: string;
  groupRunMemberEmployeeIds: string[];
  groupRunRules: EmployeeGroupRule[];
}) {
  const {
    dispatchSourceEmployeeId,
    groupRunCoordinatorEmployeeId,
    groupRunMemberEmployeeIds,
    groupRunRules,
  } = args;
  const coordinatorEmployeeId = groupRunCoordinatorEmployeeId.trim().toLowerCase();
  const normalizedDispatchSourceEmployeeId = (dispatchSourceEmployeeId || "").trim().toLowerCase();
  const memberSet = new Set(
    groupRunMemberEmployeeIds.map((value) => value.trim().toLowerCase()).filter((value) => value.length > 0),
  );
  const exactTargets = new Map<string, string>();
  const coordinatorTargets = new Map<string, string>();
  const fallbackTargets = new Map<string, string>();
  for (const rule of groupRunRules) {
    const relationType = (rule.relation_type || "").trim().toLowerCase();
    const phaseScope = (rule.phase_scope || "").trim().toLowerCase();
    if (!["delegate", "handoff"].includes(relationType)) continue;
    if (phaseScope.length > 0 && !["execute", "all", "*"].includes(phaseScope)) continue;
    const targetEmployeeId = (rule.to_employee_id || "").trim();
    const normalizedTargetEmployeeId = targetEmployeeId.toLowerCase();
    if (!targetEmployeeId || (memberSet.size > 0 && !memberSet.has(normalizedTargetEmployeeId))) {
      continue;
    }
    if (!fallbackTargets.has(normalizedTargetEmployeeId)) {
      fallbackTargets.set(normalizedTargetEmployeeId, targetEmployeeId);
    }
    const fromEmployeeId = (rule.from_employee_id || "").trim().toLowerCase();
    if (
      normalizedDispatchSourceEmployeeId &&
      fromEmployeeId === normalizedDispatchSourceEmployeeId &&
      !exactTargets.has(normalizedTargetEmployeeId)
    ) {
      exactTargets.set(normalizedTargetEmployeeId, targetEmployeeId);
    }
    if (coordinatorEmployeeId && fromEmployeeId === coordinatorEmployeeId && !coordinatorTargets.has(normalizedTargetEmployeeId)) {
      coordinatorTargets.set(normalizedTargetEmployeeId, targetEmployeeId);
    }
  }
  const preferredTargets =
    exactTargets.size > 0 ? exactTargets : coordinatorTargets.size > 0 ? coordinatorTargets : fallbackTargets;
  return {
    hasExecuteRules: fallbackTargets.size > 0,
    ids: Array.from(preferredTargets.values()),
  };
}

export function deriveGroupRunState(args: {
  groupRunSnapshot: EmployeeGroupRunSnapshot | null;
  sessionId: string;
  groupRunMemberEmployeeIds: string[];
  groupRunCoordinatorEmployeeId: string;
  groupRunRules: EmployeeGroupRule[];
}) {
  const {
    groupRunSnapshot,
    sessionId,
    groupRunMemberEmployeeIds,
    groupRunCoordinatorEmployeeId,
    groupRunRules,
  } = args;
  const groupPhaseLabelFromSnapshot = getGroupPhaseLabelFromSnapshot(groupRunSnapshot);
  const groupPhaseFromSnapshot = getGroupPhaseFromSnapshot(groupRunSnapshot);
  const groupRoundFromSnapshot = groupRunSnapshot?.current_round || 0;
  const groupReviewRound = groupRunSnapshot?.review_round || 0;
  const groupRunState = (groupRunSnapshot?.state || "").trim().toLowerCase();
  const groupWaitingLabel = groupRunSnapshot?.waiting_for_user
    ? "等待用户"
    : (groupRunSnapshot?.waiting_for_employee_id || "").trim();
  const groupStatusReason = (groupRunSnapshot?.status_reason || "").trim();
  const failedGroupRunSteps = (groupRunSnapshot?.steps || []).filter(
    (step) =>
      (step.status || "").trim().toLowerCase() === "failed" &&
      (step.step_type || "").trim().toLowerCase() === "execute",
  );
  const groupRunAssignees = Array.from(
    new Set(
      (groupRunSnapshot?.steps || [])
        .map((step) => (step.assignee_employee_id || "").trim())
        .filter((value) => value.length > 0),
    ),
  );
  const groupRunStepMap = new Map((groupRunSnapshot?.steps || []).map((step) => [step.id, step] as const));
  const latestStepReassignPayloadByStepId = (() => {
    const byStepId = new Map<string, Record<string, unknown>>();
    for (const event of groupRunSnapshot?.events || []) {
      if (event.event_type !== "step_reassigned" || !event.step_id) continue;
      byStepId.set(event.step_id, parseGroupRunEventPayload(event));
    }
    return byStepId;
  })();
  const latestGroupEventByStepId = (() => {
    const byStepId = new Map<string, EmployeeGroupRunEvent>();
    for (const event of groupRunSnapshot?.events || []) {
      if (!event.step_id) continue;
      byStepId.set(event.step_id, event);
    }
    return byStepId;
  })();
  const recentGroupEvents = (groupRunSnapshot?.events || []).slice(-4).reverse().map((event) => ({
    id: event.id,
    label: formatGroupRunEventLabel(event, groupRunStepMap),
  }));
  const groupRunEventTimelineByStepId = (() => {
    const byStepId = new Map<string, GroupRunTimelineItem[]>();
    for (const event of groupRunSnapshot?.events || []) {
      if (!event.step_id) continue;
      const label = formatGroupRunEventLabel(event, groupRunStepMap).trim();
      if (!label) continue;
      const payload = parseGroupRunEventPayload(event);
      const relatedStep = groupRunStepMap.get(event.step_id);
      const list = byStepId.get(event.step_id) || [];
      list.push({
        eventId: String(event.id || "").trim() || undefined,
        linkedSessionId: String(payload.session_id || relatedStep?.session_id || "").trim() || undefined,
        label,
        createdAt: String(event.created_at || "").trim() || undefined,
        openSessionOptions:
          event.step_id || String(payload.session_id || relatedStep?.session_id || "").trim()
            ? {
                focusHint: String(payload.output_summary || relatedStep?.output_summary || label || "").trim() || undefined,
                sourceSessionId: sessionId,
                sourceStepId: event.step_id || undefined,
                sourceEmployeeId: String(
                  payload.dispatch_source_employee_id || relatedStep?.dispatch_source_employee_id || "",
                ).trim() || undefined,
                assigneeEmployeeId: String(payload.assignee_employee_id || relatedStep?.assignee_employee_id || "").trim() || undefined,
              }
            : undefined,
      });
      byStepId.set(event.step_id, list);
    }
    for (const [stepId, items] of byStepId.entries()) {
      byStepId.set(stepId, items.slice(-3));
    }
    return byStepId;
  })();
  const groupRunExecuteStepCards: GroupRunExecuteStepCard[] = (groupRunSnapshot?.steps || [])
    .filter((step) => (step.step_type || "").trim().toLowerCase() === "execute")
    .map((step) => {
      const reassignPayload = latestStepReassignPayloadByStepId.get(step.id) || {};
      const latestEvent = latestGroupEventByStepId.get(step.id) || null;
      const latestEventPayload = latestEvent ? parseGroupRunEventPayload(latestEvent) : {};
      const currentAssigneeEmployeeId = String(reassignPayload.assignee_employee_id || step.assignee_employee_id || "").trim();
      const dispatchSourceEmployeeId = String(
        reassignPayload.dispatch_source_employee_id || step.dispatch_source_employee_id || "",
      ).trim();
      const previousAssigneeEmployeeId = String(reassignPayload.previous_assignee_employee_id || "").trim();
      const latestFailureSummary = String(
        reassignPayload.previous_output_summary ||
          ((step.status || "").trim().toLowerCase() === "failed" ? step.output_summary || step.output || "" : ""),
      ).trim();
      const attemptNo =
        typeof step.attempt_no === "number" && Number.isFinite(step.attempt_no) && step.attempt_no > 0
          ? step.attempt_no
          : 1;
      const detailSessionId = String(step.session_id || latestEventPayload.session_id || "").trim();
      const detailOutputSummary = String(step.output_summary || latestEventPayload.output_summary || step.output || "").trim();
      const latestEventCreatedAt = String(latestEvent?.created_at || "").trim();
      const sourceStepTimeline = groupRunEventTimelineByStepId.get(step.id) || [];
      const sourceStepTimelineForOpenSession = sourceStepTimeline.map(({ eventId, linkedSessionId, label, createdAt }) => ({
        eventId,
        linkedSessionId,
        label,
        createdAt,
      }));
      return {
        step,
        currentAssigneeEmployeeId,
        dispatchSourceEmployeeId,
        previousAssigneeEmployeeId,
        latestFailureSummary,
        attemptNo,
        detailSessionId,
        detailOutputSummary,
        latestEventCreatedAt,
        sourceStepTimeline: sourceStepTimeline.map((item) => ({
          ...item,
          openSessionOptions: item.linkedSessionId
            ? {
                ...(item.openSessionOptions || {}),
                focusHint: detailOutputSummary || item.label || undefined,
                sourceSessionId: sessionId,
                sourceStepId: step.id,
                sourceEmployeeId: dispatchSourceEmployeeId || undefined,
                assigneeEmployeeId: currentAssigneeEmployeeId || undefined,
                sourceStepTimeline: sourceStepTimelineForOpenSession.length > 0 ? sourceStepTimelineForOpenSession : undefined,
              }
            : item.openSessionOptions,
        })),
        detailSessionOpenOptions: detailSessionId
          ? {
              focusHint: detailOutputSummary || undefined,
              sourceSessionId: sessionId,
              sourceStepId: step.id,
              sourceEmployeeId: dispatchSourceEmployeeId || undefined,
              assigneeEmployeeId: currentAssigneeEmployeeId || undefined,
              sourceStepTimeline: sourceStepTimelineForOpenSession.length > 0 ? sourceStepTimelineForOpenSession : undefined,
            }
          : undefined,
      };
    });
  const groupMemberStatesFromSnapshot = (() => {
    const byRole = new Map<string, { status: string; stepType: string }>();
    for (const step of groupRunSnapshot?.steps || []) {
      const role = (step.assignee_employee_id || "").trim();
      if (!role) continue;
      byRole.set(role, {
        status: step.status || "running",
        stepType: (step.step_type || "").trim(),
      });
    }
    return Array.from(byRole.entries()).map(([role, info]) => ({
      role,
      status: info.status,
      stepType: info.stepType,
    }));
  })();
  const groupRunCandidateEmployeeIds = (step?: EmployeeGroupRunSnapshot["steps"][number]) =>
    Array.from(
      new Set(
        (
          getGroupRunExecuteRuleTargets({
            dispatchSourceEmployeeId: step?.dispatch_source_employee_id,
            groupRunCoordinatorEmployeeId,
            groupRunMemberEmployeeIds,
            groupRunRules,
          }).hasExecuteRules
            ? getGroupRunExecuteRuleTargets({
                dispatchSourceEmployeeId: step?.dispatch_source_employee_id,
                groupRunCoordinatorEmployeeId,
                groupRunMemberEmployeeIds,
                groupRunRules,
              }).ids
            : [...groupRunMemberEmployeeIds, ...groupRunAssignees]
        )
          .map((value) => value.trim())
          .filter((value) => value.length > 0),
      ),
    );
  const failedGroupRunReassignOptions: GroupRunReassignOption[] = failedGroupRunSteps
    .map((step) => ({
      step,
      candidateEmployeeIds: groupRunCandidateEmployeeIds(step).filter(
        (employeeId) => employeeId.trim().toLowerCase() !== (step.assignee_employee_id || "").trim().toLowerCase(),
      ),
    }))
    .filter((entry) => entry.candidateEmployeeIds.length > 0);

  return {
    groupPhaseLabelFromSnapshot,
    groupPhaseFromSnapshot,
    groupRoundFromSnapshot,
    groupReviewRound,
    groupRunState,
    groupWaitingLabel,
    groupStatusReason,
    recentGroupEvents,
    groupRunExecuteStepCards,
    groupMemberStatesFromSnapshot,
    failedGroupRunReassignOptions,
    canPauseGroupRun:
      !!groupRunSnapshot && !["paused", "done", "completed", "cancelled", "failed"].includes(groupRunState),
    canResumeGroupRun: !!groupRunSnapshot && groupRunState === "paused",
    canRetryFailedGroupRunSteps: failedGroupRunSteps.length > 0,
    canReassignFailedGroupRunStep: failedGroupRunReassignOptions.length > 0,
  };
}
