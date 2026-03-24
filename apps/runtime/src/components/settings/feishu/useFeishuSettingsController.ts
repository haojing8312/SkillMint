import { useEffect, useRef, useState } from "react";
import { openExternalUrl } from "../../../utils/openExternalUrl";
import { type SettingsTabName } from "../SettingsTabNav";
import {
  buildFeishuDiagnosticSummary,
  buildFeishuOnboardingState,
  extractFeishuInstallerQrBlock,
  getFeishuAuthorizationAction,
  getFeishuConnectionDetailSummary,
  getFeishuEnvironmentLabel,
  getFeishuRoutingStatus,
  getFeishuSetupSummary,
  getLatestInstallerOutputLine,
  resolveFeishuAuthorizationInlineError,
  resolveFeishuConnectorStatus,
  resolveFeishuGuidedInlineError,
  resolveFeishuGuidedInlineNotice,
  resolveFeishuInstallerCompletionNotice,
  resolveFeishuInstallerFlowLabel,
  resolveFeishuOnboardingPanelDisplay,
  sanitizeFeishuInstallerDisplayLines,
  shouldShowFeishuInstallerGuidedPanel,
  summarizeConnectorIssue,
  summarizeOfficialFeishuRuntimeLogs,
} from "./feishuSelectors";
import {
  approveFeishuPairingRequest as approveFeishuPairingRequestFromService,
  denyFeishuPairingRequest as denyFeishuPairingRequestFromService,
  installOpenClawLarkPlugin as installOpenClawLarkPluginFromService,
  loadFeishuAdvancedSettings as loadFeishuAdvancedSettingsFromService,
  loadFeishuGatewaySettings as loadFeishuGatewaySettingsFromService,
  loadFeishuInstallerSessionStatus as loadFeishuInstallerSessionStatusFromService,
  loadFeishuPairingRequests as loadFeishuPairingRequestsFromService,
  loadFeishuPluginChannelHosts as loadFeishuPluginChannelHostsFromService,
  loadFeishuPluginChannelSnapshot as loadFeishuPluginChannelSnapshotFromService,
  loadFeishuRuntimeStatus as loadFeishuRuntimeStatusFromService,
  loadFeishuSetupProgress as loadFeishuSetupProgressFromService,
  probeFeishuCredentials as probeFeishuCredentialsFromService,
  saveFeishuAdvancedSettings as saveFeishuAdvancedSettingsFromService,
  saveFeishuGatewaySettings as saveFeishuGatewaySettingsFromService,
  sendFeishuInstallerInput as sendFeishuInstallerInputFromService,
  startFeishuInstallerSession as startFeishuInstallerSessionFromService,
  startFeishuRuntime as startFeishuRuntimeFromService,
  stopFeishuInstallerSession as stopFeishuInstallerSessionFromService,
} from "./feishuSettingsService";
import type {
  FeishuGatewaySettings,
  FeishuPairingRequestRecord,
  FeishuPluginEnvironmentStatus,
  FeishuSetupProgress,
  OpenClawLarkInstallerMode,
  OpenClawLarkInstallerSessionStatus,
  OpenClawPluginChannelHost,
  OpenClawPluginChannelSnapshotResult,
  OpenClawPluginFeishuAdvancedSettings,
  OpenClawPluginFeishuCredentialProbeResult,
  OpenClawPluginFeishuRuntimeStatus,
} from "../../../types";

const FEISHU_OFFICIAL_PLUGIN_DOC_URL =
  "https://bytedance.larkoffice.com/docx/MFK7dDFLFoVlOGxWCv5cTXKmnMh#M0usd9GLwoiBxtx1UyjcpeMhnRe";

const DEFAULT_FEISHU_INSTALLER_SESSION: OpenClawLarkInstallerSessionStatus = {
  running: false,
  mode: null,
  started_at: null,
  last_output_at: null,
  last_error: null,
  prompt_hint: null,
  recent_output: [],
};

interface UseFeishuSettingsControllerOptions {
  activeTab: SettingsTabName;
}

function getErrorMessage(error: unknown, fallback: string): string {
  if (typeof error === "string") {
    return error || fallback;
  }
  if (error instanceof Error) {
    return error.message || fallback;
  }
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof (error as { message?: unknown }).message === "string"
  ) {
    return (error as { message: string }).message || fallback;
  }
  return fallback;
}

function normalizeFeishuGatewaySettings(settings: FeishuGatewaySettings | null | undefined): FeishuGatewaySettings {
  return {
    app_id: settings?.app_id || "",
    app_secret: settings?.app_secret || "",
    ingress_token: settings?.ingress_token || "",
    encrypt_key: settings?.encrypt_key || "",
    sidecar_base_url: settings?.sidecar_base_url || "",
  };
}

function normalizeFeishuAdvancedSettings(
  settings: OpenClawPluginFeishuAdvancedSettings | null | undefined,
): OpenClawPluginFeishuAdvancedSettings {
  return {
    groups_json: settings?.groups_json || "",
    dms_json: settings?.dms_json || "",
    footer_json: settings?.footer_json || "",
    account_overrides_json: settings?.account_overrides_json || "",
    render_mode: settings?.render_mode || "",
    streaming: settings?.streaming || "",
    text_chunk_limit: settings?.text_chunk_limit || "",
    chunk_mode: settings?.chunk_mode || "",
    reply_in_thread: settings?.reply_in_thread || "",
    group_session_scope: settings?.group_session_scope || "",
    topic_session_mode: settings?.topic_session_mode || "",
    markdown_mode: settings?.markdown_mode || "",
    markdown_table_mode: settings?.markdown_table_mode || "",
    heartbeat_visibility: settings?.heartbeat_visibility || "",
    heartbeat_interval_ms: settings?.heartbeat_interval_ms || "",
    media_max_mb: settings?.media_max_mb || "",
    http_timeout_ms: settings?.http_timeout_ms || "",
    config_writes: settings?.config_writes || "",
    webhook_host: settings?.webhook_host || "",
    webhook_port: settings?.webhook_port || "",
    dynamic_agent_creation_enabled: settings?.dynamic_agent_creation_enabled || "",
    dynamic_agent_creation_workspace_template: settings?.dynamic_agent_creation_workspace_template || "",
    dynamic_agent_creation_agent_dir_template: settings?.dynamic_agent_creation_agent_dir_template || "",
    dynamic_agent_creation_max_agents: settings?.dynamic_agent_creation_max_agents || "",
  };
}

export function useFeishuSettingsController({
  activeTab,
}: UseFeishuSettingsControllerOptions) {
  const [feishuConnectorSettings, setFeishuConnectorSettings] = useState<FeishuGatewaySettings>({
    app_id: "",
    app_secret: "",
    ingress_token: "",
    encrypt_key: "",
    sidecar_base_url: "",
  });
  const [feishuAdvancedSettings, setFeishuAdvancedSettings] = useState<OpenClawPluginFeishuAdvancedSettings>({
    groups_json: "",
    dms_json: "",
    footer_json: "",
    account_overrides_json: "",
    render_mode: "",
    streaming: "",
    text_chunk_limit: "",
    chunk_mode: "",
    reply_in_thread: "",
    group_session_scope: "",
    topic_session_mode: "",
    markdown_mode: "",
    markdown_table_mode: "",
    heartbeat_visibility: "",
    heartbeat_interval_ms: "",
    media_max_mb: "",
    http_timeout_ms: "",
    config_writes: "",
    webhook_host: "",
    webhook_port: "",
    dynamic_agent_creation_enabled: "",
    dynamic_agent_creation_workspace_template: "",
    dynamic_agent_creation_agent_dir_template: "",
    dynamic_agent_creation_max_agents: "",
  });
  const [officialFeishuRuntimeStatus, setOfficialFeishuRuntimeStatus] =
    useState<OpenClawPluginFeishuRuntimeStatus | null>(null);
  const [pluginChannelHosts, setPluginChannelHosts] = useState<OpenClawPluginChannelHost[]>([]);
  const [pluginChannelSnapshots, setPluginChannelSnapshots] =
    useState<Record<string, OpenClawPluginChannelSnapshotResult>>({});
  const [pluginChannelHostsError, setPluginChannelHostsError] = useState("");
  const [pluginChannelSnapshotsError, setPluginChannelSnapshotsError] = useState("");
  const [feishuEnvironmentStatus, setFeishuEnvironmentStatus] = useState<FeishuPluginEnvironmentStatus | null>(null);
  const [feishuSetupProgress, setFeishuSetupProgress] = useState<FeishuSetupProgress | null>(null);
  const [validatingFeishuCredentials, setValidatingFeishuCredentials] = useState(false);
  const [feishuCredentialProbe, setFeishuCredentialProbe] =
    useState<OpenClawPluginFeishuCredentialProbeResult | null>(null);
  const [feishuInstallerSession, setFeishuInstallerSession] = useState<OpenClawLarkInstallerSessionStatus>(
    DEFAULT_FEISHU_INSTALLER_SESSION,
  );
  const [feishuInstallerInput, setFeishuInstallerInput] = useState("");
  const [feishuInstallerBusy, setFeishuInstallerBusy] = useState(false);
  const [feishuInstallerStartingMode, setFeishuInstallerStartingMode] = useState<OpenClawLarkInstallerMode | null>(null);
  const handledFeishuInstallerCompletionRef = useRef("");
  const [feishuPairingRequests, setFeishuPairingRequests] = useState<FeishuPairingRequestRecord[]>([]);
  const [feishuPairingRequestsError, setFeishuPairingRequestsError] = useState("");
  const [feishuPairingActionLoading, setFeishuPairingActionLoading] = useState<"approve" | "deny" | null>(null);
  const [savingFeishuConnector, setSavingFeishuConnector] = useState(false);
  const [savingFeishuAdvancedSettings, setSavingFeishuAdvancedSettings] = useState(false);
  const [retryingFeishuConnector, setRetryingFeishuConnector] = useState(false);
  const [installingOfficialFeishuPlugin, setInstallingOfficialFeishuPlugin] = useState(false);
  const [feishuConnectorNotice, setFeishuConnectorNotice] = useState("");
  const [feishuConnectorError, setFeishuConnectorError] = useState("");
  const [feishuOnboardingPanelMode, setFeishuOnboardingPanelMode] = useState<"guided" | "skipped">("guided");
  const [feishuOnboardingSelectedPath, setFeishuOnboardingSelectedPath] = useState<
    "existing_robot" | "create_robot" | null
  >(null);
  const [feishuOnboardingSkippedSignature, setFeishuOnboardingSkippedSignature] = useState<string | null>(null);

  useEffect(() => {
    if (activeTab !== "feishu") {
      return;
    }

    void Promise.all([
      loadConnectorSettings(),
      loadConnectorStatuses(),
      loadConnectorPlatformData(),
      loadFeishuSetupProgress(),
      loadFeishuInstallerSessionStatus(),
    ]);
  }, [activeTab]);

  useEffect(() => {
    if (activeTab !== "feishu") {
      return;
    }

    const timer = window.setInterval(() => {
      void Promise.all([
        loadConnectorStatuses(),
        loadConnectorPlatformData(),
        loadFeishuSetupProgress(),
      ]);
    }, 5000);

    return () => window.clearInterval(timer);
  }, [activeTab]);

  useEffect(() => {
    if (activeTab !== "feishu" || !feishuInstallerSession.running) {
      return;
    }

    const timer = window.setInterval(() => {
      void Promise.all([
        loadFeishuInstallerSessionStatus(),
        loadConnectorSettings(),
        loadFeishuSetupProgress(),
      ]);
    }, 1500);

    return () => window.clearInterval(timer);
  }, [activeTab, feishuInstallerSession.running]);

  useEffect(() => {
    if (activeTab !== "feishu") {
      return;
    }

    const completionNotice = resolveFeishuInstallerCompletionNotice(feishuInstallerSession);
    if (!completionNotice) {
      return;
    }

    const completionKey = [
      feishuInstallerSession.mode ?? "",
      feishuInstallerSession.last_output_at ?? "",
      feishuInstallerSession.last_error ?? "",
      getLatestInstallerOutputLine(feishuInstallerSession),
    ].join("|");
    if (!completionKey || handledFeishuInstallerCompletionRef.current === completionKey) {
      return;
    }
    handledFeishuInstallerCompletionRef.current = completionKey;

    void Promise.all([
      loadConnectorSettings(),
      loadConnectorStatuses(),
      loadFeishuSetupProgress(),
    ]).finally(() => {
      setFeishuConnectorNotice(completionNotice);
    });
  }, [activeTab, feishuInstallerSession]);

  useEffect(() => {
    const onboardingState = buildFeishuOnboardingState({
      summaryState: feishuSetupProgress?.summary_state ?? null,
      setupProgress: feishuSetupProgress,
      installerMode: feishuInstallerSession?.mode ?? null,
    });
    const progressSignature = [
      onboardingState.currentStep,
      onboardingState.mode,
      onboardingState.canContinue ? "continue" : "blocked",
      onboardingState.skipped ? "backend-skipped" : "active",
    ].join("|");

    if (
      feishuOnboardingPanelMode === "skipped" &&
      feishuOnboardingSkippedSignature &&
      feishuOnboardingSkippedSignature !== progressSignature
    ) {
      setFeishuOnboardingPanelMode("guided");
      setFeishuOnboardingSkippedSignature(null);
    }
  }, [feishuOnboardingPanelMode, feishuOnboardingSkippedSignature, feishuSetupProgress, feishuInstallerSession]);

  async function loadConnectorSettings() {
    try {
      const [feishuSettings, feishuAdvanced] = await Promise.all([
        loadFeishuGatewaySettingsFromService(),
        loadFeishuAdvancedSettingsFromService(),
      ]);
      setFeishuConnectorSettings(normalizeFeishuGatewaySettings(feishuSettings));
      setFeishuAdvancedSettings(normalizeFeishuAdvancedSettings(feishuAdvanced));
    } catch (error) {
      console.warn("加载渠道连接器配置失败:", error);
    }
  }

  async function loadFeishuSetupProgress() {
    try {
      const progress = await loadFeishuSetupProgressFromService();
      if (progress) {
        setFeishuEnvironmentStatus(progress.environment ?? null);
        setFeishuSetupProgress(progress);
      } else {
        setFeishuEnvironmentStatus(null);
        setFeishuSetupProgress(null);
      }
    } catch (error) {
      console.warn("加载飞书接入进度失败:", error);
      setFeishuEnvironmentStatus(null);
      setFeishuSetupProgress(null);
    }
  }

  async function loadConnectorStatuses() {
    try {
      const runtimeStatus = await loadFeishuRuntimeStatusFromService();
      setOfficialFeishuRuntimeStatus(runtimeStatus);
    } catch (error) {
      console.warn("加载渠道连接器状态失败:", error);
      setOfficialFeishuRuntimeStatus(null);
    }
  }

  async function loadFeishuInstallerSessionStatus() {
    try {
      const status = await loadFeishuInstallerSessionStatusFromService();
      setFeishuInstallerSession(status ?? DEFAULT_FEISHU_INSTALLER_SESSION);
    } catch (error) {
      console.warn("加载飞书官方安装向导状态失败:", error);
    }
  }

  async function loadConnectorPlatformData() {
    const [hostsResult, pairingResult] = await Promise.allSettled([
      loadFeishuPluginChannelHostsFromService(),
      loadFeishuPairingRequestsFromService(),
    ]);

    const normalizedHosts =
      hostsResult.status === "fulfilled"
        ? (Array.isArray(hostsResult.value) ? hostsResult.value : []).filter(
            (host) =>
              host.channel === "feishu" ||
              host.plugin_id === "openclaw-lark" ||
              host.npm_spec === "@larksuite/openclaw-lark" ||
              host.display_name.toLowerCase().includes("feishu") ||
              host.display_name.toLowerCase().includes("lark"),
          )
        : [];
    if (hostsResult.status !== "fulfilled") {
      console.warn("加载官方插件宿主失败:", hostsResult.reason);
    }
    setPluginChannelHosts(normalizedHosts);
    setPluginChannelHostsError(hostsResult.status === "fulfilled" ? "" : "官方插件状态暂时不可用");

    if (pairingResult.status !== "fulfilled") {
      console.warn("加载飞书配对请求失败:", pairingResult.reason);
    }
    setFeishuPairingRequests(pairingResult.status === "fulfilled" && Array.isArray(pairingResult.value) ? pairingResult.value : []);
    setFeishuPairingRequestsError(pairingResult.status === "fulfilled" ? "" : "配对记录加载失败");

    if (normalizedHosts.length === 0) {
      setPluginChannelSnapshots({});
      setPluginChannelSnapshotsError("");
      return;
    }

    const snapshotResults = await Promise.allSettled(
      normalizedHosts.map((host) => loadFeishuPluginChannelSnapshotFromService(host.plugin_id)),
    );
    const nextSnapshots: Record<string, OpenClawPluginChannelSnapshotResult> = {};
    for (const result of snapshotResults) {
      if (result.status !== "fulfilled") {
        continue;
      }
      nextSnapshots[result.value.snapshot.channelId || result.value.entryPath] = result.value;
    }
    setPluginChannelSnapshots(nextSnapshots);
    setPluginChannelSnapshotsError(snapshotResults.some((result) => result.status !== "fulfilled") ? "部分账号快照暂时不可用" : "");
  }

  function applyOfficialFeishuRuntimeStatus(
    status: OpenClawPluginFeishuRuntimeStatus | null | undefined,
    options?: { showStartErrorNotice?: boolean },
  ) {
    if (!status) {
      return;
    }
    setOfficialFeishuRuntimeStatus(status);
    if (options?.showStartErrorNotice && !status.running && status.last_error?.trim()) {
      setFeishuConnectorError(`官方插件启动失败: ${status.last_error.trim()}`);
    }
  }

  function formatCompactDateTime(value: string | null | undefined) {
    const normalized = String(value || "").trim();
    if (!normalized) return "未知时间";
    const date = new Date(normalized);
    if (Number.isNaN(date.getTime())) {
      return normalized;
    }
    const year = date.getUTCFullYear();
    const month = String(date.getUTCMonth() + 1).padStart(2, "0");
    const day = String(date.getUTCDate()).padStart(2, "0");
    const hours = String(date.getUTCHours()).padStart(2, "0");
    const minutes = String(date.getUTCMinutes()).padStart(2, "0");
    return `${year}-${month}-${day} ${hours}:${minutes}`;
  }

  async function handleValidateFeishuCredentials() {
    const appId = feishuConnectorSettings.app_id.trim();
    const appSecret = feishuConnectorSettings.app_secret.trim();
    if (!appId || !appSecret) {
      setFeishuConnectorError("请先填写已有机器人的 App ID 和 App Secret");
      return;
    }

    setValidatingFeishuCredentials(true);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      const probe = await probeFeishuCredentialsFromService(appId, appSecret);
      if (!probe.ok) {
        setFeishuCredentialProbe(null);
        setFeishuConnectorError(`已有机器人校验失败: ${probe.error || "无法获取机器人信息"}`);
        return;
      }
      setFeishuCredentialProbe(probe);
      const botLabel = probe.bot_name?.trim() ? `（${probe.bot_name.trim()}）` : "";
      setFeishuConnectorNotice(`机器人信息验证成功${botLabel}`);
    } catch (error) {
      setFeishuCredentialProbe(null);
      setFeishuConnectorError("验证机器人信息失败: " + String(error));
    } finally {
      setValidatingFeishuCredentials(false);
    }
  }

  async function handleSaveFeishuConnector() {
    setSavingFeishuConnector(true);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      const saved = await saveFeishuGatewaySettingsFromService(feishuConnectorSettings);
      setFeishuConnectorSettings(saved);
      await loadConnectorStatuses();
      await loadConnectorPlatformData();
      await loadFeishuSetupProgress();
      setFeishuConnectorNotice("飞书官方插件配置已保存");
    } catch (error) {
      setFeishuConnectorError("保存飞书官方插件配置失败: " + String(error));
    } finally {
      setSavingFeishuConnector(false);
    }
  }

  async function handleSaveFeishuAdvancedSettings() {
    setSavingFeishuAdvancedSettings(true);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      const saved = await saveFeishuAdvancedSettingsFromService(feishuAdvancedSettings);
      setFeishuAdvancedSettings(saved);
      await loadConnectorPlatformData();
      await loadFeishuSetupProgress();
      setFeishuConnectorNotice("飞书高级配置已保存");
    } catch (error) {
      setFeishuConnectorError("保存飞书高级配置失败: " + String(error));
    } finally {
      setSavingFeishuAdvancedSettings(false);
    }
  }

  async function handleStartFeishuInstaller(mode: OpenClawLarkInstallerMode) {
    setFeishuInstallerBusy(true);
    setFeishuInstallerStartingMode(mode);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      if (!pluginChannelHosts.some((host) => host.status === "ready") && !feishuSetupProgress?.plugin_installed) {
        await installOpenClawLarkPluginFromService();
      }
      const status = await startFeishuInstallerSessionFromService(
        mode,
        mode === "link" ? feishuConnectorSettings.app_id.trim() : null,
        mode === "link" ? feishuConnectorSettings.app_secret.trim() : null,
      );
      setFeishuInstallerSession(status ?? DEFAULT_FEISHU_INSTALLER_SESSION);
      setFeishuInstallerInput("");
      await loadConnectorPlatformData();
      await loadFeishuSetupProgress();
      setFeishuConnectorNotice(mode === "create" ? "已启动飞书官方创建机器人向导" : "已启动飞书官方绑定机器人向导");
    } catch (error) {
      setFeishuConnectorError(
        `${mode === "create" ? "启动飞书官方创建机器人向导" : "启动飞书官方绑定机器人向导"}失败: ${String(error)}`,
      );
    } finally {
      setFeishuInstallerBusy(false);
      setFeishuInstallerStartingMode(null);
    }
  }

  async function handleSendFeishuInstallerInput() {
    const input = feishuInstallerInput.trim();
    if (!input) return;
    setFeishuInstallerBusy(true);
    setFeishuConnectorError("");
    try {
      const status = await sendFeishuInstallerInputFromService(input);
      setFeishuInstallerSession(status ?? DEFAULT_FEISHU_INSTALLER_SESSION);
      setFeishuInstallerInput("");
    } catch (error) {
      setFeishuConnectorError("发送安装向导输入失败: " + String(error));
    } finally {
      setFeishuInstallerBusy(false);
    }
  }

  async function handleStopFeishuInstallerSession() {
    setFeishuInstallerBusy(true);
    setFeishuConnectorError("");
    try {
      const status = await stopFeishuInstallerSessionFromService();
      setFeishuInstallerSession(status ?? DEFAULT_FEISHU_INSTALLER_SESSION);
      setFeishuConnectorNotice("已停止飞书官方安装向导");
    } catch (error) {
      setFeishuConnectorError("停止飞书官方安装向导失败: " + String(error));
    } finally {
      setFeishuInstallerBusy(false);
    }
  }

  async function handleRetryFeishuConnector() {
    setRetryingFeishuConnector(true);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      const runtimeStatus = await startFeishuRuntimeFromService(
        pluginChannelHosts.find((host) => host.channel === "feishu")?.plugin_id || "openclaw-lark",
        null,
      );
      if (runtimeStatus) {
        applyOfficialFeishuRuntimeStatus(runtimeStatus, { showStartErrorNotice: true });
      } else {
        await loadConnectorStatuses();
      }
      await loadConnectorPlatformData();
      await loadFeishuSetupProgress();
      setFeishuConnectorNotice(
        runtimeStatus ? (runtimeStatus.running ? "已触发飞书官方插件启动" : "已刷新飞书官方插件状态") : "已触发飞书官方插件启动",
      );
    } catch (error) {
      setFeishuConnectorError("刷新飞书官方插件状态失败: " + String(error));
    } finally {
      setRetryingFeishuConnector(false);
    }
  }

  async function handleInstallOfficialFeishuPlugin() {
    setInstallingOfficialFeishuPlugin(true);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      await installOpenClawLarkPluginFromService();
      await loadConnectorPlatformData();
      await loadFeishuSetupProgress();
      setFeishuConnectorNotice("飞书官方插件已安装");
    } catch (error) {
      setFeishuConnectorError("安装飞书官方插件失败: " + String(error));
    } finally {
      setInstallingOfficialFeishuPlugin(false);
    }
  }

  async function handleResolveFeishuPairingRequest(requestId: string, action: "approve" | "deny") {
    setFeishuPairingActionLoading(action);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      if (action === "approve") {
        await approveFeishuPairingRequestFromService(requestId);
      } else {
        await denyFeishuPairingRequestFromService(requestId);
      }
      await loadConnectorPlatformData();
      await loadFeishuSetupProgress();
      setFeishuConnectorNotice(action === "approve" ? "已批准飞书接入请求" : "已拒绝飞书接入请求");
    } catch (error) {
      setFeishuConnectorError(`${action === "approve" ? "批准" : "拒绝"}飞书接入请求失败: ${String(error)}`);
    } finally {
      setFeishuPairingActionLoading(null);
    }
  }

  async function handleInstallAndStartFeishuConnector() {
    setRetryingFeishuConnector(true);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      if (!feishuConnectorSettings.app_id.trim() || !feishuConnectorSettings.app_secret.trim()) {
        setFeishuConnectorError("请先填写并保存已有机器人的 App ID 和 App Secret");
        return;
      }

      const saved = await saveFeishuGatewaySettingsFromService(feishuConnectorSettings);
      setFeishuConnectorSettings(saved);

      if (!pluginChannelHosts.some((host) => host.status === "ready") && !feishuSetupProgress?.plugin_installed) {
        await installOpenClawLarkPluginFromService();
      }

      const runtimeStatus = await startFeishuRuntimeFromService(
        pluginChannelHosts.find((host) => host.channel === "feishu")?.plugin_id || "openclaw-lark",
        null,
      );
      if (runtimeStatus) {
        applyOfficialFeishuRuntimeStatus(runtimeStatus, { showStartErrorNotice: true });
      }
      await loadConnectorStatuses();
      await loadConnectorPlatformData();
      await loadFeishuSetupProgress();
      setFeishuConnectorNotice(runtimeStatus?.running ? "飞书连接组件已启动" : "已尝试启动飞书连接组件");
    } catch (error) {
      setFeishuConnectorError("安装并启动飞书连接失败: " + String(error));
    } finally {
      setRetryingFeishuConnector(false);
    }
  }

  async function handleOpenFeishuOfficialDocs() {
    try {
      await openExternalUrl(FEISHU_OFFICIAL_PLUGIN_DOC_URL);
    } catch (error) {
      setFeishuConnectorError(getErrorMessage(error, "打开官方文档失败，请稍后重试"));
    }
  }

  async function handleCopyFeishuDiagnostics() {
    try {
      await navigator?.clipboard?.writeText?.(
        buildFeishuDiagnosticSummary({
          connectorStatus: resolveFeishuConnectorStatus({
            running: officialFeishuRuntimeStatus?.running === true,
            lastError: officialFeishuRuntimeStatus?.last_error ?? "",
            hasInstalledOfficialFeishuPlugin:
              pluginChannelHosts.length > 0 || feishuSetupProgress?.plugin_installed === true,
          }),
          pluginVersion: feishuSetupProgress?.plugin_version || pluginChannelHosts[0]?.version || "未识别",
          defaultAccountId: Object.values(pluginChannelSnapshots)[0]?.snapshot.defaultAccountId || "未识别",
          authApproved: feishuSetupProgress?.auth_status === "approved",
          defaultRoutingEmployeeName: feishuSetupProgress?.default_routing_employee_name || "未设置",
          scopedRoutingCount: feishuSetupProgress?.scoped_routing_count ?? 0,
          lastEventAtLabel: formatCompactDateTime(officialFeishuRuntimeStatus?.last_event_at),
          connectionDetailSummary: getFeishuConnectionDetailSummary({
            connectorStatus: resolveFeishuConnectorStatus({
              running: officialFeishuRuntimeStatus?.running === true,
              lastError: officialFeishuRuntimeStatus?.last_error ?? "",
              hasInstalledOfficialFeishuPlugin:
                pluginChannelHosts.length > 0 || feishuSetupProgress?.plugin_installed === true,
            }),
            runtimeRunning: officialFeishuRuntimeStatus?.running === true,
            authApproved: feishuSetupProgress?.auth_status === "approved",
            defaultRoutingEmployeeName: feishuSetupProgress?.default_routing_employee_name,
            scopedRoutingCount: feishuSetupProgress?.scoped_routing_count,
          }),
          recentLogsSummary: summarizeOfficialFeishuRuntimeLogs(officialFeishuRuntimeStatus),
        }),
      );
      setFeishuConnectorNotice("连接诊断摘要已复制");
    } catch (error) {
      setFeishuConnectorError(getErrorMessage(error, "复制连接诊断摘要失败，请稍后重试"));
    }
  }

  async function handleRefreshFeishuSetup() {
    setRetryingFeishuConnector(true);
    setFeishuConnectorNotice("");
    setFeishuConnectorError("");
    try {
      await Promise.all([
        loadConnectorSettings(),
        loadConnectorStatuses(),
        loadConnectorPlatformData(),
        loadFeishuSetupProgress(),
      ]);
    } catch (error) {
      setFeishuConnectorError("刷新飞书接入状态失败: " + String(error));
    } finally {
      setRetryingFeishuConnector(false);
    }
  }

  return {
    feishuConnectorSettings,
    setFeishuConnectorSettings,
    feishuAdvancedSettings,
    setFeishuAdvancedSettings,
    officialFeishuRuntimeStatus,
    setOfficialFeishuRuntimeStatus,
    pluginChannelHosts,
    setPluginChannelHosts,
    pluginChannelSnapshots,
    setPluginChannelSnapshots,
    pluginChannelHostsError,
    setPluginChannelHostsError,
    pluginChannelSnapshotsError,
    setPluginChannelSnapshotsError,
    feishuEnvironmentStatus,
    setFeishuEnvironmentStatus,
    feishuSetupProgress,
    setFeishuSetupProgress,
    validatingFeishuCredentials,
    setValidatingFeishuCredentials,
    feishuCredentialProbe,
    setFeishuCredentialProbe,
    feishuInstallerSession,
    setFeishuInstallerSession,
    feishuInstallerInput,
    setFeishuInstallerInput,
    feishuInstallerBusy,
    setFeishuInstallerBusy,
    feishuInstallerStartingMode,
    setFeishuInstallerStartingMode,
    handledFeishuInstallerCompletionRef,
    feishuPairingRequests,
    setFeishuPairingRequests,
    feishuPairingRequestsError,
    setFeishuPairingRequestsError,
    feishuPairingActionLoading,
    setFeishuPairingActionLoading,
    savingFeishuConnector,
    setSavingFeishuConnector,
    savingFeishuAdvancedSettings,
    setSavingFeishuAdvancedSettings,
    retryingFeishuConnector,
    setRetryingFeishuConnector,
    installingOfficialFeishuPlugin,
    setInstallingOfficialFeishuPlugin,
    feishuConnectorNotice,
    setFeishuConnectorNotice,
    feishuConnectorError,
    setFeishuConnectorError,
    feishuOnboardingPanelMode,
    setFeishuOnboardingPanelMode,
    feishuOnboardingSelectedPath,
    setFeishuOnboardingSelectedPath,
    feishuOnboardingSkippedSignature,
    setFeishuOnboardingSkippedSignature,
    loadConnectorSettings,
    loadFeishuSetupProgress,
    loadConnectorStatuses,
    loadFeishuInstallerSessionStatus,
    loadConnectorPlatformData,
    applyOfficialFeishuRuntimeStatus,
    formatCompactDateTime,
    handleValidateFeishuCredentials,
    handleSaveFeishuConnector,
    handleSaveFeishuAdvancedSettings,
    handleStartFeishuInstaller,
    handleSendFeishuInstallerInput,
    handleStopFeishuInstallerSession,
    handleRetryFeishuConnector,
    handleInstallOfficialFeishuPlugin,
    handleResolveFeishuPairingRequest,
    handleInstallAndStartFeishuConnector,
    handleOpenFeishuOfficialDocs,
    handleCopyFeishuDiagnostics,
    handleRefreshFeishuSetup,
  };
}
