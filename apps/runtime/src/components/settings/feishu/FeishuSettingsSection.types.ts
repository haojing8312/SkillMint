import type {
  FeishuGatewaySettings,
  FeishuPairingRequestRecord,
  FeishuPluginEnvironmentStatus,
  FeishuSetupProgress,
  OpenClawLarkInstallerMode,
  OpenClawLarkInstallerSessionStatus,
  OpenClawPluginFeishuCredentialProbeResult,
} from "../../../types";
import type { FeishuOnboardingPanelDisplay, FeishuOnboardingState, FeishuOnboardingStep } from "./feishuOnboardingHelpers";

export type FeishuAuthorizationAction = {
  label: string;
  busyLabel: string;
};

export type FeishuRoutingStatus = {
  label: string;
  description: string;
  actionLabel: string;
};

export type FeishuSetupSummary = {
  title: string;
  description: string;
};

export interface FeishuSettingsSectionProps {
  onOpenEmployees?: () => void;
  feishuConnectorSettings: FeishuGatewaySettings;
  onUpdateFeishuConnectorSettings: (patch: Partial<FeishuGatewaySettings>) => void;
  feishuEnvironmentStatus: FeishuPluginEnvironmentStatus | null;
  feishuSetupProgress: FeishuSetupProgress | null;
  validatingFeishuCredentials: boolean;
  feishuCredentialProbe: OpenClawPluginFeishuCredentialProbeResult | null;
  feishuInstallerSession: OpenClawLarkInstallerSessionStatus;
  feishuInstallerInput: string;
  onUpdateFeishuInstallerInput: (value: string) => void;
  feishuInstallerBusy: boolean;
  feishuInstallerStartingMode: OpenClawLarkInstallerMode | null;
  feishuPairingActionLoading: "approve" | "deny" | null;
  savingFeishuConnector: boolean;
  retryingFeishuConnector: boolean;
  installingOfficialFeishuPlugin: boolean;
  feishuConnectorNotice: string;
  feishuConnectorError: string;
  feishuOnboardingState: FeishuOnboardingState;
  feishuOnboardingPanelMode: "guided" | "skipped";
  feishuOnboardingSelectedPath: "existing_robot" | "create_robot" | null;
  feishuOnboardingSkippedSignature: string | null;
  onOpenFeishuOnboardingPath: (path: "existing_robot" | "create_robot") => void;
  onReopenFeishuOnboarding: () => void;
  onSkipFeishuOnboarding: (signature: string) => void;
  feishuOnboardingProgressSignature: string;
  feishuOnboardingIsSkipped: boolean;
  feishuOnboardingEffectiveBranch: "existing_robot" | "create_robot" | null;
  feishuOnboardingHeaderStep: FeishuOnboardingStep;
  feishuOnboardingHeaderMode: "existing_robot" | "create_robot";
  feishuOnboardingPanelDisplay: FeishuOnboardingPanelDisplay;
  showFeishuInstallerGuidedPanel: boolean;
  feishuGuidedInlineError: string | null;
  feishuGuidedInlineNotice: string | null;
  feishuAuthorizationInlineError: string | null;
  feishuInstallerDisplayMode: OpenClawLarkInstallerMode | null;
  feishuInstallerFlowLabel: string;
  feishuInstallerQrBlock: string[];
  feishuInstallerDisplayLines: string[];
  feishuInstallerStartupHint: string | null;
  feishuAuthorizationAction: FeishuAuthorizationAction;
  feishuRoutingStatus: FeishuRoutingStatus;
  feishuRoutingActionAvailable: boolean;
  feishuOnboardingPrimaryActionLabel: string;
  feishuOnboardingPrimaryActionDisabled: boolean;
  feishuSetupSummary: FeishuSetupSummary;
  pendingFeishuPairingCount: number;
  pendingFeishuPairingRequest: FeishuPairingRequestRecord | null;
  getFeishuEnvironmentLabel: (ready: boolean, fallback: string) => string;
  formatCompactDateTime: (value: string | null | undefined) => string;
  handleRefreshFeishuSetup: () => Promise<void>;
  handleOpenFeishuOfficialDocs: () => Promise<void>;
  handleValidateFeishuCredentials: () => Promise<void>;
  handleSaveFeishuConnector: () => Promise<void>;
  handleInstallOfficialFeishuPlugin: () => Promise<void>;
  handleInstallAndStartFeishuConnector: () => Promise<void>;
  handleResolveFeishuPairingRequest: (requestId: string, action: "approve" | "deny") => Promise<void>;
  handleStartFeishuInstaller: (mode: "create" | "link") => Promise<void>;
  handleStopFeishuInstallerSession: () => Promise<void>;
  handleSendFeishuInstallerInput: () => Promise<void>;
}
