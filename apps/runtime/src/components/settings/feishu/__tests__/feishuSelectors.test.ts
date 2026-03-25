import { describe, expect, it } from "vitest";
import {
  buildFeishuDiagnosticsClipboardText,
  getFeishuConnectionDetailSummary,
  resolveFeishuConnectorStatus,
} from "../feishuSelectors";

describe("feishuSelectors", () => {
  it("summarizes pending pairing approvals as a normal connection state", () => {
    const connectorStatus = resolveFeishuConnectorStatus({
      running: true,
      lastError: null,
      hasInstalledOfficialFeishuPlugin: true,
    });

    expect(
      getFeishuConnectionDetailSummary({
        connectorStatus,
        runtimeRunning: true,
        authApproved: true,
        pendingPairings: 1,
        defaultRoutingEmployeeName: "太子",
        scopedRoutingCount: 0,
      }),
    ).toBe("连接正常，但有新的接入请求等待批准。");
  });

  it("includes pending pairing approvals in the copied diagnostics summary", () => {
    const summary = buildFeishuDiagnosticsClipboardText({
      connectorStatus: {
        running: true,
        lastError: null,
        hasInstalledOfficialFeishuPlugin: true,
      },
      pluginVersion: "2026.3.25",
      defaultAccountId: "default",
      authApproved: true,
      pendingPairings: 1,
      defaultRoutingEmployeeName: "太子",
      scopedRoutingCount: 0,
      lastEventAt: "2026-03-24T15:06:00Z",
      runtimeStatus: {
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: true,
        started_at: "2026-03-24T15:00:00Z",
        last_stop_at: null,
        last_event_at: "2026-03-24T15:06:00Z",
        last_error: null,
        pid: 1234,
        port: 3100,
        recent_logs: [
          "[info] runtime: feishu[default]: sender ou_sender not paired, creating pairing request",
          "[pairing] feishu: created request pairing-1 for ou_sender code=6X4ZN54W",
        ],
      },
      pluginChannelHosts: 1,
      pluginInstalled: true,
    });

    expect(summary).toContain("诊断摘要: 连接正常，但有新的接入请求等待批准。");
  });
});
