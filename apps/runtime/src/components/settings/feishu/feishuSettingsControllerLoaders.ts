import type {
  OpenClawPluginChannelHost,
  OpenClawPluginFeishuRuntimeStatus,
} from "../../../types";
import type { FeishuSettingsControllerActionDeps } from "./feishuSettingsControllerActionTypes";
import {
  loadFeishuAdvancedSettings as loadFeishuAdvancedSettingsFromService,
  loadFeishuGatewaySettings as loadFeishuGatewaySettingsFromService,
  loadFeishuPairingRequests as loadFeishuPairingRequestsFromService,
  loadFeishuPluginChannelHosts as loadFeishuPluginChannelHostsFromService,
  loadFeishuPluginChannelSnapshot as loadFeishuPluginChannelSnapshotFromService,
  normalizeFeishuAdvancedSettings,
  normalizeFeishuGatewaySettings,
} from "./feishuSettingsService";

function normalizeFeishuHosts(hosts: OpenClawPluginChannelHost[]) {
  return hosts.filter(
    (host) =>
      host.channel === "feishu" ||
      host.plugin_id === "openclaw-lark" ||
      host.npm_spec === "@larksuite/openclaw-lark" ||
      host.display_name.toLowerCase().includes("feishu") ||
      host.display_name.toLowerCase().includes("lark"),
  );
}

export function createFeishuSettingsControllerLoaders(deps: FeishuSettingsControllerActionDeps) {
  async function loadConnectorSettings() {
    try {
      const [feishuSettings, feishuAdvanced] = await Promise.all([
        loadFeishuGatewaySettingsFromService(),
        loadFeishuAdvancedSettingsFromService(),
      ]);
      deps.setFeishuConnectorSettings(normalizeFeishuGatewaySettings(feishuSettings));
      deps.setFeishuAdvancedSettings(normalizeFeishuAdvancedSettings(feishuAdvanced));
    } catch (error) {
      console.warn("加载渠道连接器配置失败:", error);
    }
  }

  async function loadConnectorPlatformData() {
    const [hostsResult, pairingResult] = await Promise.allSettled([
      loadFeishuPluginChannelHostsFromService(),
      loadFeishuPairingRequestsFromService(),
    ]);

    const normalizedHosts =
      hostsResult.status === "fulfilled"
        ? normalizeFeishuHosts(Array.isArray(hostsResult.value) ? hostsResult.value : [])
        : [];
    if (hostsResult.status !== "fulfilled") {
      console.warn("加载官方插件宿主失败:", hostsResult.reason);
    }
    deps.setPluginChannelHosts(normalizedHosts);
    deps.setPluginChannelHostsError(hostsResult.status === "fulfilled" ? "" : "官方插件状态暂时不可用");

    if (pairingResult.status !== "fulfilled") {
      console.warn("加载飞书配对请求失败:", pairingResult.reason);
    }
    deps.setFeishuPairingRequests(
      pairingResult.status === "fulfilled" && Array.isArray(pairingResult.value) ? pairingResult.value : [],
    );
    deps.setFeishuPairingRequestsError(pairingResult.status === "fulfilled" ? "" : "配对记录加载失败");

    if (normalizedHosts.length === 0) {
      deps.setPluginChannelSnapshots({});
      deps.setPluginChannelSnapshotsError("");
      return;
    }

    const snapshotResults = await Promise.allSettled(
      normalizedHosts.map((host) => loadFeishuPluginChannelSnapshotFromService(host.plugin_id)),
    );
    const nextSnapshots: Record<string, Awaited<ReturnType<typeof loadFeishuPluginChannelSnapshotFromService>>> = {};
    for (const result of snapshotResults) {
      if (result.status !== "fulfilled") {
        continue;
      }
      nextSnapshots[result.value.snapshot.channelId || result.value.entryPath] = result.value;
    }
    deps.setPluginChannelSnapshots(nextSnapshots);
    deps.setPluginChannelSnapshotsError(
      snapshotResults.some((result) => result.status !== "fulfilled") ? "部分账号快照暂时不可用" : "",
    );
  }

  function applyOfficialFeishuRuntimeStatus(
    status: OpenClawPluginFeishuRuntimeStatus | null | undefined,
    options?: { showStartErrorNotice?: boolean },
  ) {
    if (!status) {
      return;
    }
    deps.setOfficialFeishuRuntimeStatus(status);
    if (options?.showStartErrorNotice && !status.running && status.last_error?.trim()) {
      deps.setFeishuConnectorError(`官方插件启动失败: ${status.last_error.trim()}`);
    }
  }

  return {
    loadConnectorSettings,
    loadConnectorPlatformData,
    applyOfficialFeishuRuntimeStatus,
  };
}
