import { RiskConfirmDialog } from "../RiskConfirmDialog";

interface ChatActionDialogsProps {
  approvalOpen: boolean;
  approvalDialog:
    | {
        title: string;
        summary: string;
        impact?: string;
        note?: string;
        irreversible?: boolean;
      }
    | null
    | undefined;
  approvalLoading: boolean;
  onAllowOnce: () => void;
  onAllowAlways: () => void;
  onDeny: () => void;
  installOpen: boolean;
  installSummary: string;
  installImpact?: string;
  installLoading: boolean;
  onConfirmInstall: () => void;
  onCancelInstall: () => void;
}

export function ChatActionDialogs({
  approvalOpen,
  approvalDialog,
  approvalLoading,
  onAllowOnce,
  onAllowAlways,
  onDeny,
  installOpen,
  installSummary,
  installImpact,
  installLoading,
  onConfirmInstall,
  onCancelInstall,
}: ChatActionDialogsProps) {
  return (
    <>
      <RiskConfirmDialog
        open={approvalOpen}
        level="high"
        title={approvalDialog?.title || "高危操作确认"}
        summary={approvalDialog?.summary || "请确认是否继续执行。"}
        impact={approvalDialog?.impact}
        note={approvalDialog?.note}
        irreversible={approvalDialog?.irreversible}
        confirmLabel="允许一次"
        secondaryActionLabel="始终允许"
        cancelLabel="取消"
        loading={approvalLoading}
        onConfirm={onAllowOnce}
        onSecondaryAction={onAllowAlways}
        onCancel={onDeny}
      />
      <RiskConfirmDialog
        open={installOpen}
        level="medium"
        title="安装技能"
        summary={installSummary}
        impact={installImpact}
        irreversible={false}
        confirmLabel="确认安装"
        cancelLabel="取消"
        loading={installLoading}
        onConfirm={onConfirmInstall}
        onCancel={onCancelInstall}
      />
    </>
  );
}
