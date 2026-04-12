import type {
  FeishuGatewaySettings,
  FeishuPairingRequestRecord,
  FeishuPluginEnvironmentStatus,
  FeishuSetupProgress,
  OpenClawLarkInstallerMode,
  OpenClawLarkInstallerSessionStatus,
  OpenClawPluginFeishuCredentialProbeResult,
  OpenClawPluginFeishuRuntimeStatus,
} from "../../../types";
import type {
  FeishuAuthorizationAction,
  FeishuRoutingStatus,
} from "./FeishuSettingsSection.types";

export interface FeishuAdvancedConsoleSectionProps {
  onOpenEmployees?: () => void;
  feishuConnectorSettings: FeishuGatewaySettings;
  onUpdateFeishuConnectorSettings: (patch: Partial<FeishuGatewaySettings>) => void;
  feishuEnvironmentStatus: FeishuPluginEnvironmentStatus | null;
  feishuSetupProgress: FeishuSetupProgress | null;
  officialFeishuRuntimeStatus: OpenClawPluginFeishuRuntimeStatus | null;
  feishuCredentialProbe: OpenClawPluginFeishuCredentialProbeResult | null;
  validatingFeishuCredentials: boolean;
  savingFeishuConnector: boolean;
  retryingFeishuConnector: boolean;
  installingOfficialFeishuPlugin: boolean;
  feishuInstallerSession: OpenClawLarkInstallerSessionStatus;
  feishuInstallerInput: string;
  onUpdateFeishuInstallerInput: (value: string) => void;
  feishuInstallerBusy: boolean;
  feishuInstallerStartingMode: OpenClawLarkInstallerMode | null;
  feishuPairingActionLoading: "approve" | "deny" | null;
  pendingFeishuPairingCount: number;
  pendingFeishuPairingRequest: FeishuPairingRequestRecord | null;
  feishuOnboardingEffectiveBranch: "existing_robot" | "create_robot" | null;
  feishuAuthorizationInlineError: string | null;
  feishuOnboardingHeaderStep: string;
  feishuInstallerDisplayMode: OpenClawLarkInstallerMode | null;
  feishuInstallerStartupHint: string | null;
  feishuAuthorizationAction: FeishuAuthorizationAction;
  feishuRoutingStatus: FeishuRoutingStatus;
  getFeishuEnvironmentLabel: (ready: boolean, fallback: string) => string;
  formatCompactDateTime: (value: string | null | undefined) => string;
  handleValidateFeishuCredentials: () => Promise<void>;
  handleSaveFeishuConnector: () => Promise<void>;
  handleInstallAndStartFeishuConnector: () => Promise<void>;
  handleRefreshFeishuSetup: () => Promise<void>;
  handleResolveFeishuPairingRequest: (requestId: string, action: "approve" | "deny") => Promise<void>;
  handleStartFeishuInstaller: (mode: "create" | "link") => Promise<void>;
  handleStopFeishuInstallerSession: () => Promise<void>;
  handleSendFeishuInstallerInput: () => Promise<void>;
}
