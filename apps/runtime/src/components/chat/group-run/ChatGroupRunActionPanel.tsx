import type { EmployeeGroupRunSnapshot } from "../../../types";
import type { GroupRunActionLoading, GroupRunMemberState, GroupRunReassignOption } from "./chatGroupRunBoardShared";

type ChatGroupRunActionPanelProps = {
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
};

export function ChatGroupRunActionPanel({
  groupRunSnapshot,
  onApproveGroupRunReview,
  onRejectGroupRunReview,
  onPauseGroupRun,
  onResumeGroupRun,
  onRetryFailedGroupRunSteps,
  onReassignFailedGroupRunStep,
  groupRunActionLoading,
  canPauseGroupRun,
  canResumeGroupRun,
  canRetryFailedGroupRunSteps,
  canReassignFailedGroupRunStep,
  failedGroupRunReassignOptions,
  groupMemberStates,
}: ChatGroupRunActionPanelProps) {
  return (
    <>
      {groupRunSnapshot && (groupRunSnapshot.state || "").trim().toLowerCase() === "waiting_review" && (
        <div className="mt-2 flex items-center gap-2">
          <button
            type="button"
            data-testid="group-run-review-reject"
            onClick={() => void onRejectGroupRunReview()}
            disabled={groupRunActionLoading !== null}
            className="rounded bg-rose-600 px-2.5 py-1 text-[11px] text-white hover:bg-rose-700 disabled:bg-rose-300"
          >
            {groupRunActionLoading === "reject" ? "打回中..." : "打回重审"}
          </button>
          <button
            type="button"
            data-testid="group-run-review-approve"
            onClick={() => void onApproveGroupRunReview()}
            disabled={groupRunActionLoading !== null}
            className="rounded bg-emerald-600 px-2.5 py-1 text-[11px] text-white hover:bg-emerald-700 disabled:bg-emerald-300"
          >
            {groupRunActionLoading === "approve" ? "通过中..." : "通过审议"}
          </button>
        </div>
      )}
      {groupRunSnapshot && (
        <div className="mt-2 flex flex-wrap items-center gap-2">
          {canPauseGroupRun && (
            <button
              type="button"
              data-testid="group-run-pause"
              onClick={() => void onPauseGroupRun()}
              disabled={groupRunActionLoading !== null}
              className="rounded bg-slate-600 px-2.5 py-1 text-[11px] text-white hover:bg-slate-700 disabled:bg-slate-300"
            >
              {groupRunActionLoading === "pause" ? "暂停中..." : "暂停协作"}
            </button>
          )}
          {canResumeGroupRun && (
            <button
              type="button"
              data-testid="group-run-resume"
              onClick={() => void onResumeGroupRun()}
              disabled={groupRunActionLoading !== null}
              className="rounded bg-sky-600 px-2.5 py-1 text-[11px] text-white hover:bg-sky-700 disabled:bg-sky-300"
            >
              {groupRunActionLoading === "resume" ? "继续中..." : "继续协作"}
            </button>
          )}
          {canRetryFailedGroupRunSteps && (
            <button
              type="button"
              data-testid="group-run-retry-failed"
              onClick={() => void onRetryFailedGroupRunSteps()}
              disabled={groupRunActionLoading !== null}
              className="rounded bg-amber-600 px-2.5 py-1 text-[11px] text-white hover:bg-amber-700 disabled:bg-amber-300"
            >
              {groupRunActionLoading === "retry" ? "重试中..." : "重试失败步骤"}
            </button>
          )}
          {canReassignFailedGroupRunStep && (
            <div className="w-full space-y-1.5">
              {failedGroupRunReassignOptions.map(({ step, candidateEmployeeIds }) => (
                <div
                  key={step.id}
                  data-testid={`group-run-reassign-row-${step.id}`}
                  className="rounded border border-indigo-200 bg-white/70 px-2.5 py-2"
                >
                  <div className="text-[11px] font-medium text-indigo-800">{`失败步骤：${step.assignee_employee_id || step.id}`}</div>
                  {(step.dispatch_source_employee_id || "").trim().length > 0 && (
                    <div className="mt-1 text-[10px] text-indigo-700/80">{`来源：${step.dispatch_source_employee_id}`}</div>
                  )}
                  {(step.output || "").trim().length > 0 && (
                    <div className="mt-1 text-[10px] text-indigo-700/80">{step.output}</div>
                  )}
                  <div className="mt-1.5 flex flex-wrap gap-2">
                    {candidateEmployeeIds.map((employeeId) => (
                      <button
                        key={`${step.id}-${employeeId}`}
                        type="button"
                        data-testid={`group-run-reassign-${step.id}-${employeeId}`}
                        onClick={() => void onReassignFailedGroupRunStep(step.id, employeeId)}
                        disabled={groupRunActionLoading !== null}
                        className="rounded bg-fuchsia-600 px-2.5 py-1 text-[11px] text-white hover:bg-fuchsia-700 disabled:bg-fuchsia-300"
                      >
                        {groupRunActionLoading === "reassign" ? "改派中..." : `改派给${employeeId}`}
                      </button>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
      {groupMemberStates.length > 0 && (
        <div className="mt-2 space-y-1">
          {groupMemberStates.map((member) => (
            <div key={member.role} className="text-[11px] text-indigo-800">
              {member.role}
              {member.stepType ? ` · ${member.stepType}` : ""}
              {` · ${member.status}`}
            </div>
          ))}
        </div>
      )}
    </>
  );
}
