import type { OpenClawPluginFeishuAdvancedSettings } from "../../../types";

export type FeishuAdvancedFieldConfig = {
  key: keyof OpenClawPluginFeishuAdvancedSettings;
  label: string;
  description: string;
  kind: "input" | "textarea";
  rows?: number;
};

export interface FeishuAdvancedSectionProps {
  connectionDetailSummary: string;
  feishuAdvancedSettings: OpenClawPluginFeishuAdvancedSettings;
  onUpdateFeishuAdvancedSettings: (patch: Partial<OpenClawPluginFeishuAdvancedSettings>) => void;
  connectionStatusLabel: string;
  pluginVersionLabel: string;
  currentAccountLabel: string;
  pendingPairingCount: number;
  lastEventAtLabel: string;
  recentIssueLabel: string;
  runtimeLogsLabel: string;
  retryingFeishuConnector: boolean;
  savingFeishuAdvancedSettings: boolean;
  onRefreshFeishuSetup: () => Promise<void>;
  onSaveFeishuAdvancedSettings: () => Promise<void>;
  onCopyDiagnostics: () => Promise<void>;
}
