import { FeishuAdvancedSettingsForm } from "./FeishuAdvancedSettingsForm";
import { FeishuConnectionDetailsPanel } from "./FeishuConnectionDetailsPanel";
import type { FeishuAdvancedSectionProps } from "./FeishuAdvancedSection.types";

export type { FeishuAdvancedSectionProps } from "./FeishuAdvancedSection.types";

export function FeishuAdvancedSection(props: FeishuAdvancedSectionProps) {
  return (
    <>
      <FeishuConnectionDetailsPanel
        connectionDetailSummary={props.connectionDetailSummary}
        connectionStatusLabel={props.connectionStatusLabel}
        pluginVersionLabel={props.pluginVersionLabel}
        currentAccountLabel={props.currentAccountLabel}
        pendingPairingCount={props.pendingPairingCount}
        lastEventAtLabel={props.lastEventAtLabel}
        recentIssueLabel={props.recentIssueLabel}
        runtimeLogsLabel={props.runtimeLogsLabel}
        retryingFeishuConnector={props.retryingFeishuConnector}
        onRefreshFeishuSetup={props.onRefreshFeishuSetup}
        onCopyDiagnostics={props.onCopyDiagnostics}
      />
      <FeishuAdvancedSettingsForm
        feishuAdvancedSettings={props.feishuAdvancedSettings}
        onUpdateFeishuAdvancedSettings={props.onUpdateFeishuAdvancedSettings}
        savingFeishuAdvancedSettings={props.savingFeishuAdvancedSettings}
        onSaveFeishuAdvancedSettings={props.onSaveFeishuAdvancedSettings}
      />
    </>
  );
}
