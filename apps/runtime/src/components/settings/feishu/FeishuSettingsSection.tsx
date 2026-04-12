import type { FeishuSettingsSectionProps } from "./FeishuSettingsSection.types";
import { FeishuOnboardingCard } from "./FeishuOnboardingCard";
import { FeishuSettingsOverview } from "./FeishuSettingsOverview";
export type { FeishuSettingsSectionProps } from "./FeishuSettingsSection.types";

export function FeishuSettingsSection({
  onOpenEmployees,
  feishuConnectorSettings,
  onUpdateFeishuConnectorSettings,
  feishuEnvironmentStatus,
  feishuSetupProgress,
  validatingFeishuCredentials,
  feishuCredentialProbe,
  feishuInstallerSession,
  feishuInstallerInput,
  onUpdateFeishuInstallerInput,
  feishuInstallerBusy,
  feishuInstallerStartingMode,
  feishuPairingActionLoading,
  savingFeishuConnector,
  retryingFeishuConnector,
  installingOfficialFeishuPlugin,
  feishuConnectorNotice,
  feishuConnectorError,
  feishuOnboardingState,
  feishuOnboardingPanelMode,
  feishuOnboardingSelectedPath,
  feishuOnboardingSkippedSignature,
  onOpenFeishuOnboardingPath,
  onReopenFeishuOnboarding,
  onSkipFeishuOnboarding,
  feishuOnboardingProgressSignature,
  feishuOnboardingIsSkipped,
  feishuOnboardingEffectiveBranch,
  feishuOnboardingHeaderStep,
  feishuOnboardingHeaderMode,
  feishuOnboardingPanelDisplay,
  showFeishuInstallerGuidedPanel,
  feishuGuidedInlineError,
  feishuGuidedInlineNotice,
  feishuAuthorizationInlineError,
  feishuInstallerDisplayMode,
  feishuInstallerFlowLabel,
  feishuInstallerQrBlock,
  feishuInstallerDisplayLines,
  feishuInstallerStartupHint,
  feishuAuthorizationAction,
  feishuRoutingStatus,
  feishuRoutingActionAvailable,
  feishuOnboardingPrimaryActionLabel,
  feishuOnboardingPrimaryActionDisabled,
  feishuSetupSummary,
  pendingFeishuPairingCount,
  pendingFeishuPairingRequest,
  getFeishuEnvironmentLabel,
  formatCompactDateTime,
  handleRefreshFeishuSetup,
  handleOpenFeishuOfficialDocs,
  handleValidateFeishuCredentials,
  handleSaveFeishuConnector,
  handleInstallOfficialFeishuPlugin,
  handleInstallAndStartFeishuConnector,
  handleResolveFeishuPairingRequest,
  handleStartFeishuInstaller,
  handleStopFeishuInstallerSession,
  handleSendFeishuInstallerInput,
}: FeishuSettingsSectionProps) {
  return (
    <div data-testid="connector-panel-feishu" className="space-y-3">
      <div className="bg-white rounded-lg p-4 space-y-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div className="space-y-1">
            <div className="text-sm font-medium text-gray-900">飞书连接</div>
            <div className="text-xs text-gray-500">先完成机器人接入，再安装飞书官方插件并完成授权，最后补充接待员工设置。</div>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => void handleRefreshFeishuSetup()}
              disabled={retryingFeishuConnector}
              className="h-8 px-3 rounded border border-gray-200 bg-white text-xs text-gray-700 hover:bg-gray-50 disabled:bg-gray-100"
            >
              {retryingFeishuConnector ? "检测中..." : "重新检测"}
            </button>
            <button
              type="button"
              onClick={() => void handleOpenFeishuOfficialDocs()}
              className="inline-flex h-8 items-center rounded border border-blue-200 bg-blue-50 px-3 text-xs text-blue-700 hover:bg-blue-100"
            >
              查看官方文档
            </button>
          </div>
        </div>

        {feishuConnectorError && !feishuGuidedInlineError && !feishuAuthorizationInlineError ? (
          <div className="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">{feishuConnectorError}</div>
        ) : null}
        {feishuConnectorNotice && !feishuGuidedInlineNotice ? (
          <div className="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-700">{feishuConnectorNotice}</div>
        ) : null}

        <FeishuSettingsOverview
          feishuSetupSummary={feishuSetupSummary}
          feishuEnvironmentStatus={feishuEnvironmentStatus}
          feishuSetupProgress={feishuSetupProgress}
          feishuRoutingStatus={feishuRoutingStatus}
        />
        <FeishuOnboardingCard
          onOpenEmployees={onOpenEmployees}
          feishuInstallerSession={feishuInstallerSession}
          feishuInstallerBusy={feishuInstallerBusy}
          feishuInstallerStartingMode={feishuInstallerStartingMode}
          feishuPairingActionLoading={feishuPairingActionLoading}
          retryingFeishuConnector={retryingFeishuConnector}
          feishuOnboardingState={feishuOnboardingState}
          onOpenFeishuOnboardingPath={onOpenFeishuOnboardingPath}
          onReopenFeishuOnboarding={onReopenFeishuOnboarding}
          onSkipFeishuOnboarding={onSkipFeishuOnboarding}
          feishuOnboardingProgressSignature={feishuOnboardingProgressSignature}
          feishuOnboardingIsSkipped={feishuOnboardingIsSkipped}
          feishuOnboardingEffectiveBranch={feishuOnboardingEffectiveBranch}
          feishuOnboardingHeaderStep={feishuOnboardingHeaderStep}
          feishuOnboardingHeaderMode={feishuOnboardingHeaderMode}
          feishuOnboardingPanelDisplay={feishuOnboardingPanelDisplay}
          showFeishuInstallerGuidedPanel={showFeishuInstallerGuidedPanel}
          feishuGuidedInlineError={feishuGuidedInlineError}
          feishuGuidedInlineNotice={feishuGuidedInlineNotice}
          feishuInstallerDisplayMode={feishuInstallerDisplayMode}
          feishuInstallerFlowLabel={feishuInstallerFlowLabel}
          feishuInstallerQrBlock={feishuInstallerQrBlock}
          feishuInstallerDisplayLines={feishuInstallerDisplayLines}
          feishuInstallerStartupHint={feishuInstallerStartupHint}
          feishuOnboardingPrimaryActionLabel={feishuOnboardingPrimaryActionLabel}
          feishuOnboardingPrimaryActionDisabled={feishuOnboardingPrimaryActionDisabled}
          pendingFeishuPairingRequest={pendingFeishuPairingRequest}
          formatCompactDateTime={formatCompactDateTime}
          handleRefreshFeishuSetup={handleRefreshFeishuSetup}
          handleValidateFeishuCredentials={handleValidateFeishuCredentials}
          handleInstallOfficialFeishuPlugin={handleInstallOfficialFeishuPlugin}
          handleInstallAndStartFeishuConnector={handleInstallAndStartFeishuConnector}
          handleResolveFeishuPairingRequest={handleResolveFeishuPairingRequest}
          handleStartFeishuInstaller={handleStartFeishuInstaller}
          handleStopFeishuInstallerSession={handleStopFeishuInstallerSession}
        />
      </div>
    </div>
  );
}
