import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import type {
  ChatDelegationCardState,
  EmployeeGroupRule,
  EmployeeGroupRunSnapshot,
  ImRoleDispatchRequest,
  ImRoleTimelineEvent,
  Message,
} from "../../types";
import {
  continueEmployeeGroupRun,
  getEmployeeGroupRunSnapshot,
  listEmployeeGroupRules,
  listEmployeeGroups,
  pauseEmployeeGroupRun,
  reassignGroupRunStep,
  resumeEmployeeGroupRun,
  retryEmployeeGroupRunFailedSteps,
  reviewGroupRunStep,
} from "../../services/chat/chatGroupRunService";

type GroupRunAction =
  | "approve"
  | "reject"
  | "pause"
  | "resume"
  | "retry"
  | "reassign"
  | null;

type UseChatCollaborationControllerArgs = {
  sessionId: string;
  mainRoleName: string;
  getCurrentMainRoleName: () => string;
  onMainRoleNameChange: (roleName: string) => void;
  onMainSummaryDeliveredChange: (delivered: boolean) => void;
  onDelegationCardsChange: (updater: (prev: ChatDelegationCardState[]) => ChatDelegationCardState[]) => void;
  onMessagesAppend: (message: Message) => void;
  onResetForSessionSwitch: () => void;
};

export function useChatCollaborationController({
  sessionId,
  mainRoleName,
  getCurrentMainRoleName,
  onMainRoleNameChange,
  onMainSummaryDeliveredChange,
  onDelegationCardsChange,
  onMessagesAppend,
  onResetForSessionSwitch,
}: UseChatCollaborationControllerArgs) {
  const [imRoleEvents, setImRoleEvents] = useState<ImRoleTimelineEvent[]>([]);
  const [groupRunSnapshot, setGroupRunSnapshot] = useState<EmployeeGroupRunSnapshot | null>(null);
  const [groupRunMemberEmployeeIds, setGroupRunMemberEmployeeIds] = useState<string[]>([]);
  const [groupRunCoordinatorEmployeeId, setGroupRunCoordinatorEmployeeId] = useState("");
  const [groupRunRules, setGroupRunRules] = useState<EmployeeGroupRule[]>([]);
  const [expandedGroupRunStepIds, setExpandedGroupRunStepIds] = useState<string[]>([]);
  const [groupRunActionLoading, setGroupRunActionLoading] = useState<GroupRunAction>(null);
  const seenDispatchRequestKeysRef = useRef<Set<string>>(new Set());

  const resetForSessionSwitch = () => {
    seenDispatchRequestKeysRef.current = new Set();
    setImRoleEvents([]);
    setGroupRunSnapshot(null);
    setGroupRunMemberEmployeeIds([]);
    setGroupRunCoordinatorEmployeeId("");
    setGroupRunRules([]);
    setExpandedGroupRunStepIds([]);
    onResetForSessionSwitch();
  };

  useEffect(() => {
    const unlistenPromise = listen<ImRoleTimelineEvent>("im-role-event", ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      setImRoleEvents((prev) => [...prev, payload]);
      const roleLabel = (payload.role_name || payload.role_id || "").trim();
      if (payload.sender_role === "main_agent" && roleLabel) {
        onMainRoleNameChange(roleLabel);
      }
      if (payload.sender_role === "main_agent") {
        if (payload.status === "completed") {
          onMainSummaryDeliveredChange(true);
        } else if (payload.status === "running") {
          onMainSummaryDeliveredChange(false);
        }
      }
      if (
        payload.sender_role === "sub_agent" &&
        roleLabel &&
        (payload.status === "completed" || payload.status === "failed")
      ) {
        onDelegationCardsChange((prev) => {
          const next = [...prev];
          let matchedIndex = -1;
          for (let index = next.length - 1; index >= 0; index -= 1) {
            const item = next[index];
            const byTaskId = payload.task_id && item.taskId === payload.task_id;
            const byRole = item.toRole === roleLabel;
            if (item.status === "running" && (byTaskId || byRole)) {
              matchedIndex = index;
              break;
            }
          }
          if (matchedIndex >= 0) {
            next[matchedIndex] = {
              ...next[matchedIndex],
              status: payload.status === "failed" ? "failed" : "completed",
            };
          }
          return next;
        });
      }
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [onDelegationCardsChange, onMainRoleNameChange, onMainSummaryDeliveredChange, sessionId]);

  useEffect(() => {
    const unlistenPromise = listen<ImRoleDispatchRequest>("im-role-dispatch-request", ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      const dispatchKey = [
        payload.session_id,
        payload.thread_id,
        payload.task_id || "",
        payload.role_id || "",
        payload.prompt || "",
      ].join("::");
      if (seenDispatchRequestKeysRef.current.has(dispatchKey)) {
        return;
      }
      seenDispatchRequestKeysRef.current.add(dispatchKey);
      const cleanPrompt = (payload.prompt || "")
        .replace(/@_[A-Za-z0-9_]+/g, " ")
        .replace(/\s+/g, " ")
        .trim();
      const roleLabel = payload.role_name || payload.role_id;
      const dispatchedMessage: Message = {
        role: "user",
        content: `【${roleLabel}】${cleanPrompt || payload.prompt || ""}`,
        created_at: new Date().toISOString(),
      };
      setTimeout(() => {
        onMessagesAppend(dispatchedMessage);
      }, 0);
      setImRoleEvents((prev) => [
        ...prev,
        {
          session_id: payload.session_id,
          thread_id: payload.thread_id,
          role_id: payload.role_id,
          role_name: roleLabel,
          sender_role: payload.sender_role ?? "main_agent",
          sender_employee_id: payload.sender_employee_id ?? payload.role_id,
          target_employee_id: payload.target_employee_id ?? payload.role_id,
          task_id: payload.task_id,
          parent_task_id: payload.parent_task_id,
          message_type: payload.message_type ?? "delegate_request",
          source_channel: payload.source_channel ?? "app",
          status: "running",
          summary: `任务已分发(${payload.agent_type}) -> ${roleLabel}`,
        },
      ]);
      const delegationId = (payload.task_id || "").trim() || `${payload.thread_id}-${Date.now()}`;
      onMainSummaryDeliveredChange(false);
      onDelegationCardsChange((prev) => {
        const next = prev.filter((item) => item.id !== delegationId);
        next.push({
          id: delegationId,
          fromRole: getCurrentMainRoleName() || mainRoleName || "主员工",
          toRole: roleLabel,
          status: "running",
          taskId: payload.task_id,
        });
        return next.slice(-8);
      });
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [
    getCurrentMainRoleName,
    mainRoleName,
    onDelegationCardsChange,
    onMainSummaryDeliveredChange,
    onMessagesAppend,
    sessionId,
  ]);

  useEffect(() => {
    let disposed = false;
    const loadSnapshot = async () => {
      try {
        const snapshot = await getEmployeeGroupRunSnapshot(sessionId);
        if (!disposed) {
          setGroupRunSnapshot(snapshot ?? null);
        }
      } catch {
        if (!disposed) {
          setGroupRunSnapshot(null);
        }
      }
    };
    void loadSnapshot();
    const timer = setInterval(() => {
      void loadSnapshot();
    }, 3000);
    return () => {
      disposed = true;
      clearInterval(timer);
    };
  }, [sessionId]);

  useEffect(() => {
    let disposed = false;
    const groupId = (groupRunSnapshot?.group_id || "").trim();
    if (!groupId) {
      setGroupRunMemberEmployeeIds([]);
      setGroupRunCoordinatorEmployeeId("");
      setGroupRunRules([]);
      return () => {
        disposed = true;
      };
    }
    const loadGroupMembers = async () => {
      try {
        const [groups, rules] = await Promise.all([listEmployeeGroups(), listEmployeeGroupRules(groupId)]);
        if (disposed) return;
        const matchedGroup = groups.find((group) => (group.id || "").trim() === groupId) ?? null;
        const memberIds = (matchedGroup?.member_employee_ids || [])
          .map((value) => (value || "").trim())
          .filter((value) => value.length > 0);
        setGroupRunMemberEmployeeIds(memberIds);
        setGroupRunCoordinatorEmployeeId((matchedGroup?.coordinator_employee_id || "").trim());
        setGroupRunRules(rules);
      } catch {
        if (!disposed) {
          setGroupRunMemberEmployeeIds([]);
          setGroupRunCoordinatorEmployeeId("");
          setGroupRunRules([]);
        }
      }
    };
    void loadGroupMembers();
    return () => {
      disposed = true;
    };
  }, [groupRunSnapshot?.group_id]);

  const refreshGroupRunSnapshot = async (targetSessionId?: string) => {
    const snapshotSessionId = (targetSessionId || groupRunSnapshot?.session_id || sessionId || "").trim();
    if (!snapshotSessionId) return;
    const snapshot = await getEmployeeGroupRunSnapshot(snapshotSessionId);
    if (snapshot) {
      setGroupRunSnapshot(snapshot);
    }
  };

  const handleApproveGroupRunReview = async () => {
    if (!groupRunSnapshot?.run_id || groupRunActionLoading) return;
    setGroupRunActionLoading("approve");
    try {
      await reviewGroupRunStep(groupRunSnapshot.run_id, "approve", "前端确认通过");
      const snapshot = await continueEmployeeGroupRun(groupRunSnapshot.run_id);
      setGroupRunSnapshot(snapshot);
    } catch (error) {
      console.error("审核通过失败:", error);
    } finally {
      setGroupRunActionLoading(null);
    }
  };

  const handleRejectGroupRunReview = async () => {
    if (!groupRunSnapshot?.run_id || groupRunActionLoading) return;
    setGroupRunActionLoading("reject");
    try {
      await reviewGroupRunStep(groupRunSnapshot.run_id, "reject", "前端要求补充方案");
      const snapshot = await continueEmployeeGroupRun(groupRunSnapshot.run_id);
      setGroupRunSnapshot(snapshot);
    } catch (error) {
      console.error("审核打回失败:", error);
    } finally {
      setGroupRunActionLoading(null);
    }
  };

  const handlePauseGroupRun = async () => {
    if (!groupRunSnapshot?.run_id || groupRunActionLoading) return;
    setGroupRunActionLoading("pause");
    try {
      await pauseEmployeeGroupRun(groupRunSnapshot.run_id, "前端人工暂停");
      await refreshGroupRunSnapshot(groupRunSnapshot.session_id);
    } catch (error) {
      console.error("暂停协作失败:", error);
    } finally {
      setGroupRunActionLoading(null);
    }
  };

  const handleResumeGroupRun = async () => {
    if (!groupRunSnapshot?.run_id || groupRunActionLoading) return;
    setGroupRunActionLoading("resume");
    try {
      await resumeEmployeeGroupRun(groupRunSnapshot.run_id);
      const snapshot = await continueEmployeeGroupRun(groupRunSnapshot.run_id);
      setGroupRunSnapshot(snapshot);
    } catch (error) {
      console.error("继续协作失败:", error);
    } finally {
      setGroupRunActionLoading(null);
    }
  };

  const handleRetryFailedGroupRunSteps = async () => {
    if (!groupRunSnapshot?.run_id || groupRunActionLoading) return;
    setGroupRunActionLoading("retry");
    try {
      await retryEmployeeGroupRunFailedSteps(groupRunSnapshot.run_id);
      await refreshGroupRunSnapshot(groupRunSnapshot.session_id);
    } catch (error) {
      console.error("重试失败步骤失败:", error);
    } finally {
      setGroupRunActionLoading(null);
    }
  };

  const handleReassignFailedGroupRunStep = async (stepId: string, assigneeEmployeeId: string) => {
    if (!groupRunSnapshot?.run_id || groupRunActionLoading) return;
    setGroupRunActionLoading("reassign");
    try {
      await reassignGroupRunStep(stepId, assigneeEmployeeId);
      const snapshot = await continueEmployeeGroupRun(groupRunSnapshot.run_id);
      setGroupRunSnapshot(snapshot);
    } catch (error) {
      console.error("改派失败步骤失败:", error);
    } finally {
      setGroupRunActionLoading(null);
    }
  };

  return {
    imRoleEvents,
    groupRunSnapshot,
    groupRunMemberEmployeeIds,
    groupRunCoordinatorEmployeeId,
    groupRunRules,
    expandedGroupRunStepIds,
    setExpandedGroupRunStepIds,
    groupRunActionLoading,
    resetForSessionSwitch,
    handleApproveGroupRunReview,
    handleRejectGroupRunReview,
    handlePauseGroupRun,
    handleResumeGroupRun,
    handleRetryFailedGroupRunSteps,
    handleReassignFailedGroupRunStep,
  };
}
