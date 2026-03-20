import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SettingsView } from "../SettingsView";

const invokeMock = vi.fn();

type InvokeOverride = (payload?: Record<string, unknown>) => Promise<unknown>;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function installInvokeMock(overrides: Record<string, InvokeOverride> = {}) {
  invokeMock.mockReset();
  invokeMock.mockImplementation((command: string, payload?: Record<string, unknown>) => {
    const override = overrides[command];
    if (override) {
      return override(payload);
    }
    if (command === "list_model_configs") return Promise.resolve([]);
    if (command === "list_mcp_servers") return Promise.resolve([]);
    if (command === "list_search_configs") return Promise.resolve([]);
    if (command === "get_runtime_preferences") {
      return Promise.resolve({
        default_work_dir: "",
        default_language: "zh-CN",
        immersive_translation_enabled: true,
        immersive_translation_display: "translated_only",
        immersive_translation_trigger: "auto",
        translation_engine: "model_then_free",
        translation_model_id: "",
        launch_at_login: false,
        launch_minimized: false,
        close_to_tray: true,
      });
    }
    if (command === "get_desktop_lifecycle_paths") {
      return Promise.resolve({
        app_data_dir: "",
        cache_dir: "",
        log_dir: "",
        default_work_dir: "",
      });
    }
    if (command === "get_routing_settings") {
      return Promise.resolve({ max_call_depth: 4, node_timeout_seconds: 60, retry_count: 0 });
    }
    if (command === "list_builtin_provider_plugins") return Promise.resolve([]);
    if (command === "list_provider_configs") return Promise.resolve([]);
    if (command === "get_capability_routing_policy") return Promise.resolve(null);
    if (command === "list_capability_route_templates") return Promise.resolve([]);
    if (command === "get_feishu_gateway_settings") {
      return Promise.resolve({
        app_id: "cli-app",
        app_secret: "cli-secret",
        ingress_token: "",
        encrypt_key: "",
        sidecar_base_url: "",
      });
    }
    if (command === "get_openclaw_plugin_feishu_advanced_settings") {
      return Promise.resolve({
        groups_json: '{\n  "oc_demo": {\n    "enabled": true,\n    "requireMention": true\n  }\n}',
        dms_json: '{\n  "ou_demo": {\n    "enabled": true,\n    "systemPrompt": "优先回答测试问题"\n  }\n}',
        footer_json: '{\n  "status": true,\n  "elapsed": true\n}',
        account_overrides_json: '{\n  "default": {\n    "renderMode": "card"\n  }\n}',
        render_mode: "card",
        streaming: "true",
        text_chunk_limit: "2400",
        chunk_mode: "newline",
        reply_in_thread: "enabled",
        group_session_scope: "group_sender",
        topic_session_mode: "enabled",
        markdown_mode: "native",
        markdown_table_mode: "native",
        heartbeat_visibility: "visible",
        heartbeat_interval_ms: "30000",
        media_max_mb: "20",
        http_timeout_ms: "60000",
        config_writes: "true",
        webhook_host: "127.0.0.1",
        webhook_port: "8787",
        dynamic_agent_creation_enabled: "true",
        dynamic_agent_creation_workspace_template: "workspace/{sender_id}",
        dynamic_agent_creation_agent_dir_template: "agents/{sender_id}",
        dynamic_agent_creation_max_agents: "48",
      });
    }
    if (command === "set_openclaw_plugin_feishu_advanced_settings") {
      const settings = (payload?.settings as Record<string, string> | undefined) ?? {};
      return Promise.resolve({
        groups_json: settings.groups_json ?? "",
        dms_json: settings.dms_json ?? "",
        footer_json: settings.footer_json ?? "",
        account_overrides_json: settings.account_overrides_json ?? "",
        render_mode: settings.render_mode ?? "",
        streaming: settings.streaming ?? "",
        text_chunk_limit: settings.text_chunk_limit ?? "",
        chunk_mode: settings.chunk_mode ?? "",
        reply_in_thread: settings.reply_in_thread ?? "",
        group_session_scope: settings.group_session_scope ?? "",
        topic_session_mode: settings.topic_session_mode ?? "",
        markdown_mode: settings.markdown_mode ?? "",
        markdown_table_mode: settings.markdown_table_mode ?? "",
        heartbeat_visibility: settings.heartbeat_visibility ?? "",
        heartbeat_interval_ms: settings.heartbeat_interval_ms ?? "",
        media_max_mb: settings.media_max_mb ?? "",
        http_timeout_ms: settings.http_timeout_ms ?? "",
        config_writes: settings.config_writes ?? "",
        webhook_host: settings.webhook_host ?? "",
        webhook_port: settings.webhook_port ?? "",
        dynamic_agent_creation_enabled: settings.dynamic_agent_creation_enabled ?? "",
        dynamic_agent_creation_workspace_template:
          settings.dynamic_agent_creation_workspace_template ?? "",
        dynamic_agent_creation_agent_dir_template:
          settings.dynamic_agent_creation_agent_dir_template ?? "",
        dynamic_agent_creation_max_agents:
          settings.dynamic_agent_creation_max_agents ?? "",
      });
    }
    if (command === "get_feishu_long_connection_status") {
      return Promise.resolve({
        running: false,
        started_at: null,
        queued_events: 0,
      });
    }
    if (command === "get_openclaw_plugin_feishu_runtime_status") {
      return Promise.resolve({
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: false,
        started_at: null,
        last_stop_at: null,
        last_event_at: null,
        last_error: null,
        pid: null,
        port: null,
        recent_logs: [],
      });
    }
    if (command === "get_openclaw_lark_installer_session_status") {
      return Promise.resolve({
        running: false,
        mode: null,
        started_at: null,
        last_output_at: null,
        last_error: null,
        prompt_hint: null,
        recent_output: [],
      });
    }
    if (command === "start_openclaw_lark_installer_session") {
      return Promise.resolve({
        running: true,
        mode: payload?.mode || "link",
        started_at: "2026-03-19T10:00:00Z",
        last_output_at: "2026-03-19T10:00:01Z",
        last_error: null,
        prompt_hint: "请输入机器人 App ID",
        recent_output: ["[system] official installer started"],
      });
    }
    if (command === "send_openclaw_lark_installer_input") {
      return Promise.resolve({
        running: true,
        mode: "link",
        started_at: "2026-03-19T10:00:00Z",
        last_output_at: "2026-03-19T10:00:02Z",
        last_error: null,
        prompt_hint: null,
        recent_output: ["[manual-input] cli-app"],
      });
    }
    if (command === "stop_openclaw_lark_installer_session") {
      return Promise.resolve({
        running: false,
        mode: "link",
        started_at: "2026-03-19T10:00:00Z",
        last_output_at: "2026-03-19T10:00:03Z",
        last_error: null,
        prompt_hint: null,
        recent_output: ["[system] official installer finished"],
      });
    }
    if (command === "probe_openclaw_plugin_feishu_credentials") {
      return Promise.resolve({
        ok: true,
        app_id: payload?.appId || "cli-app",
        bot_name: "WorkClaw Bot",
        bot_open_id: "ou_bot_open_id",
      });
    }
    if (command === "list_channel_connectors") {
      return Promise.resolve([
        {
          channel: "feishu",
          display_name: "飞书连接器",
          capabilities: ["receive_text", "send_text", "group_route", "direct_route"],
        },
        {
          channel: "wecom",
          display_name: "企业微信连接器",
          capabilities: ["receive_text", "send_text", "group_route", "direct_route"],
        },
      ]);
    }
    if (command === "list_openclaw_plugin_channel_hosts") {
      return Promise.resolve([
        {
          plugin_id: "openclaw-lark",
          npm_spec: "@larksuite/openclaw-lark",
          version: "2026.3.17",
          channel: "feishu",
          display_name: "Feishu",
          capabilities: ["media", "reactions", "threads", "outbound", "pairing"],
          reload_config_prefixes: ["channels.feishu"],
          target_hint: "<chatId|user:openId|chat:chatId>",
          docs_path: "/channels/feishu",
          status: "ready",
          error: null,
        },
      ]);
    }
    if (command === "get_openclaw_plugin_feishu_channel_snapshot") {
      return Promise.resolve({
        pluginRoot: "D:/plugins/openclaw-lark",
        preparedRoot: "D:/runtime/.workclaw-plugin-host-fixtures/openclaw-lark",
        manifest: {},
        entryPath: "D:/plugins/openclaw-lark/index.js",
        snapshot: {
          channelId: "feishu",
          defaultAccountId: "default",
          accountIds: ["default"],
          accounts: [
            {
              accountId: "default",
              account: {
                accountId: "default",
                enabled: true,
                configured: true,
              },
              describedAccount: {
                accountId: "default",
                enabled: true,
                configured: true,
              },
              allowFrom: [],
              warnings: [],
            },
          ],
          reloadConfigPrefixes: ["channels.feishu"],
          targetHint: "<chatId|user:openId|chat:chatId>",
        },
        logRecordCount: 1,
      });
    }
    if (command === "list_feishu_pairing_requests") {
      return Promise.resolve([
        {
          id: "pairing-1",
          channel: "feishu",
          account_id: "default",
          sender_id: "ou_applicant",
          chat_id: "ou_applicant",
          code: "PAIR1234",
          status: "pending",
          created_at: "2026-03-19T10:00:00Z",
          updated_at: "2026-03-19T10:00:00Z",
          resolved_at: null,
          resolved_by_user: "",
        },
      ]);
    }
    if (command === "approve_feishu_pairing_request" || command === "deny_feishu_pairing_request") {
      return Promise.resolve({
        id: "pairing-1",
        channel: "feishu",
        account_id: "default",
        sender_id: "ou_applicant",
        chat_id: "ou_applicant",
        code: "PAIR1234",
        status: command === "approve_feishu_pairing_request" ? "approved" : "denied",
        created_at: "2026-03-19T10:00:00Z",
        updated_at: "2026-03-19T10:01:00Z",
        resolved_at: "2026-03-19T10:01:00Z",
        resolved_by_user: "settings-ui",
      });
    }
    if (command === "get_channel_connector_diagnostics") {
      const instanceId = payload?.instanceId;
      if (instanceId === "feishu:default") {
        return Promise.resolve({
          connector: {
            channel: "feishu",
            display_name: "飞书连接器",
            capabilities: ["receive_text", "send_text", "group_route", "direct_route"],
          },
          status: "stopped",
          health: {
            adapter_name: "feishu",
            instance_id: "feishu:default",
            state: "stopped",
            last_ok_at: null,
            last_error: null,
            reconnect_attempts: 0,
            queue_depth: 0,
            issue: null,
          },
          replay: {
            retained_events: 0,
            acked_events: 0,
          },
        });
      }
      return Promise.resolve(null);
    }
    return Promise.resolve(null);
  });
}

describe("SettingsView connector visibility", () => {
  beforeEach(() => {
    installInvokeMock();
  });

  test("hides wecom connector panel and diagnostics on settings page", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByTestId("connector-panel-feishu")).toBeInTheDocument();
    });

    expect(screen.queryByTestId("connector-panel-wecom")).not.toBeInTheDocument();
    expect(screen.queryByText("企业微信连接器")).not.toBeInTheDocument();
    expect(screen.queryByText("企业微信连接异常")).not.toBeInTheDocument();
    expect(screen.queryByPlaceholderText("企业微信 Corp ID")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "连接配置" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "官方插件" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "配对与授权" })).toBeInTheDocument();
    expect(screen.getByText("飞书接入概览")).toBeInTheDocument();
    expect(screen.getAllByText("飞书官方插件配置").length).toBeGreaterThan(0);
    expect(screen.getByPlaceholderText("飞书事件订阅 Verification Token")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("飞书事件订阅 Encrypt Key")).toBeInTheDocument();
    expect(screen.getByText(/官方插件当前默认按 websocket 模式运行/)).toBeInTheDocument();
    expect(screen.getAllByText("未启动").length).toBeGreaterThan(0);
    expect(screen.getByText("飞书官方插件配置")).toBeInTheDocument();
    expect(screen.queryByText("连接器诊断")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "官方插件" }));
    expect(screen.getByText("OpenClaw 官方插件频道宿主")).toBeInTheDocument();
    expect(screen.getAllByText("@larksuite/openclaw-lark").length).toBeGreaterThan(0);
    expect(screen.getByText("官方插件账号视图 · 默认 default")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "配对与授权" }));
    expect(screen.getByText("待处理配对请求")).toBeInTheDocument();
    expect(screen.getByText(/PAIR1234/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "批准" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "拒绝" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "连接配置" }));
    expect(screen.getByText("配置模式")).toBeInTheDocument();
    expect(invokeMock.mock.calls.some(([command]) => command === "get_feishu_long_connection_status")).toBe(false);
  });

  test("shows official plugin install wizard entry point and doc link", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "重新运行安装向导" })).toBeInTheDocument();
    });

    expect(screen.getByText("飞书官方插件安装向导")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "查看官方文档" })).toHaveAttribute(
      "href",
      "https://bytedance.larkoffice.com/docx/MFK7dDFLFoVlOGxWCv5cTXKmnMh#M0usd9GLwoiBxtx1UyjcpeMhnRe",
    );

    fireEvent.click(screen.getByRole("button", { name: "关联已有机器人" }));

    expect(
      screen.getAllByText((_, node) => node?.textContent?.includes("/feishu start") ?? false).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getAllByText((_, node) => node?.textContent?.includes("/feishu auth") ?? false).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getAllByText((_, node) => node?.textContent?.includes("/feishu doctor") ?? false).length,
    ).toBeGreaterThan(0);
  });

  test("switches official plugin onboarding modes between create and link", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "新建机器人" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "新建机器人" }));
    expect(screen.getByText("目标体验应与官方安装器一致：安装时直接创建机器人，完成后自动启动插件运行态。")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "关联已有机器人" }));
    expect(screen.getByText(/请在下方填写已有机器人的/)).toBeInTheDocument();
  });

  test("starts official installer session for create flow and renders session output", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "运行新建机器人向导" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "运行新建机器人向导" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("start_openclaw_lark_installer_session", {
        mode: "create",
        appId: null,
        appSecret: null,
      });
    });

    expect(screen.getByText("官方安装会话 · 运行中")).toBeInTheDocument();
    expect(screen.getByText(/\[system\] official installer started/)).toBeInTheDocument();
    expect(screen.getByText("请输入机器人 App ID")).toBeInTheDocument();
  });

  test("links existing bot through direct credential probe and runtime start", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "运行关联已有机器人向导" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "运行关联已有机器人向导" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("probe_openclaw_plugin_feishu_credentials", {
        appId: "cli-app",
        appSecret: "cli-secret",
      });
    });

    expect(invokeMock).toHaveBeenCalledWith("set_feishu_gateway_settings", {
      settings: expect.objectContaining({
        app_id: "cli-app",
        app_secret: "cli-secret",
      }),
    });
    expect(invokeMock).toHaveBeenCalledWith("start_openclaw_plugin_feishu_runtime", {
      pluginId: "openclaw-lark",
      accountId: null,
    });
    expect(invokeMock).not.toHaveBeenCalledWith("start_openclaw_lark_installer_session", {
      mode: "link",
      appId: "cli-app",
      appSecret: "cli-secret",
    });

    await waitFor(() => {
      expect(screen.getByText(/已完成已有机器人校验并启动飞书官方插件/)).toBeInTheDocument();
    });
  });

  test("resolves feishu pairing requests from settings actions", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "配对与授权" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "配对与授权" }));
    fireEvent.click(screen.getByRole("button", { name: "批准" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("approve_feishu_pairing_request", {
        requestId: "pairing-1",
        resolvedByUser: "settings-ui",
      });
    });
  });

  test("saves feishu settings and starts official plugin runtime when plugin mode is ready", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "保存官方插件配置" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "保存官方插件配置" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("set_feishu_gateway_settings", {
        settings: expect.objectContaining({
          app_id: "cli-app",
          app_secret: "cli-secret",
        }),
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/飞书官方插件配置已保存/)).toBeInTheDocument();
    });

    expect(invokeMock).toHaveBeenCalledWith("start_openclaw_plugin_feishu_runtime", {
      pluginId: "openclaw-lark",
      accountId: null,
    });
    expect(invokeMock).not.toHaveBeenCalledWith("start_feishu_long_connection", expect.anything());
  });

  test("starts official plugin runtime instead of legacy sidecar when retrying in plugin mode", async () => {
    installInvokeMock({
      start_feishu_long_connection: async () => {
        throw new Error("sidecar offline");
      },
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "刷新插件状态" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "刷新插件状态" }));

    await waitFor(() => {
      expect(screen.getByText(/已触发飞书官方插件启动/)).toBeInTheDocument();
    });

    expect(invokeMock).toHaveBeenCalledWith("start_openclaw_plugin_feishu_runtime", {
      pluginId: "openclaw-lark",
      accountId: null,
    });
    expect(invokeMock).not.toHaveBeenCalledWith("start_feishu_long_connection", expect.anything());
  });

  test("loads and saves official feishu advanced json settings", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByText("飞书官方插件配置")).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("get_openclaw_plugin_feishu_advanced_settings");
    });

    const groupsEditor = await screen.findByLabelText("群聊高级规则 JSON");
    const dmsEditor = screen.getByLabelText("私聊高级规则 JSON");
    const footerEditor = screen.getByLabelText("回复页脚 JSON");
    const accountOverridesEditor = screen.getByLabelText("账号覆盖 JSON");
    const renderModeInput = screen.getByLabelText("渲染模式");
    const streamingInput = screen.getByLabelText("流式输出");
    const textChunkLimitInput = screen.getByLabelText("文本分块上限");
    const chunkModeInput = screen.getByLabelText("分块模式");
    const replyInThreadInput = screen.getByLabelText("线程内回复");
    const groupSessionScopeInput = screen.getByLabelText("群聊会话范围");
    const topicSessionModeInput = screen.getByLabelText("话题会话模式");
    const markdownModeInput = screen.getByLabelText("Markdown 模式");
    const heartbeatIntervalInput = screen.getByLabelText("心跳间隔毫秒");
    const webhookHostInput = screen.getByLabelText("Webhook Host");
    const configWritesInput = screen.getByLabelText("允许插件写回配置");
    const dynamicWorkspaceTemplateInput = screen.getByLabelText("动态工作区模板");

    expect(String((groupsEditor as HTMLTextAreaElement).value)).toContain("\"oc_demo\"");
    expect(String((dmsEditor as HTMLTextAreaElement).value)).toContain("\"ou_demo\"");
    expect(String((footerEditor as HTMLTextAreaElement).value)).toContain("\"elapsed\"");
    expect(String((accountOverridesEditor as HTMLTextAreaElement).value)).toContain("\"renderMode\"");
    expect(renderModeInput).toHaveValue("card");
    expect(streamingInput).toHaveValue("true");
    expect(textChunkLimitInput).toHaveValue("2400");
    expect(chunkModeInput).toHaveValue("newline");
    expect(replyInThreadInput).toHaveValue("enabled");
    expect(groupSessionScopeInput).toHaveValue("group_sender");
    expect(topicSessionModeInput).toHaveValue("enabled");
    expect(markdownModeInput).toHaveValue("native");
    expect(heartbeatIntervalInput).toHaveValue("30000");
    expect(webhookHostInput).toHaveValue("127.0.0.1");
    expect(configWritesInput).toHaveValue("true");
    expect(dynamicWorkspaceTemplateInput).toHaveValue("workspace/{sender_id}");

    fireEvent.change(groupsEditor, {
      target: {
        value: '{\n  "oc_ops": {\n    "enabled": true,\n    "requireMention": false\n  }\n}',
      },
    });
    fireEvent.change(renderModeInput, { target: { value: "raw" } });
    fireEvent.change(streamingInput, { target: { value: "false" } });
    fireEvent.change(textChunkLimitInput, { target: { value: "3200" } });
    fireEvent.change(chunkModeInput, { target: { value: "length" } });
    fireEvent.change(replyInThreadInput, { target: { value: "disabled" } });
    fireEvent.change(groupSessionScopeInput, { target: { value: "group" } });
    fireEvent.change(topicSessionModeInput, { target: { value: "disabled" } });
    fireEvent.change(markdownModeInput, { target: { value: "rich" } });
    fireEvent.change(heartbeatIntervalInput, { target: { value: "15000" } });
    fireEvent.change(webhookHostInput, { target: { value: "localhost" } });
    fireEvent.change(configWritesInput, { target: { value: "false" } });
    fireEvent.change(dynamicWorkspaceTemplateInput, {
      target: { value: "employees/{sender_id}" },
    });

    fireEvent.click(screen.getByRole("button", { name: "保存高级配置" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("set_openclaw_plugin_feishu_advanced_settings", {
        settings: {
          groups_json: '{\n  "oc_ops": {\n    "enabled": true,\n    "requireMention": false\n  }\n}',
          dms_json: '{\n  "ou_demo": {\n    "enabled": true,\n    "systemPrompt": "优先回答测试问题"\n  }\n}',
          footer_json: '{\n  "status": true,\n  "elapsed": true\n}',
          account_overrides_json: '{\n  "default": {\n    "renderMode": "card"\n  }\n}',
          render_mode: "raw",
          streaming: "false",
          text_chunk_limit: "3200",
          chunk_mode: "length",
          reply_in_thread: "disabled",
          group_session_scope: "group",
          topic_session_mode: "disabled",
          markdown_mode: "rich",
          markdown_table_mode: "native",
          heartbeat_visibility: "visible",
          heartbeat_interval_ms: "15000",
          media_max_mb: "20",
          http_timeout_ms: "60000",
          config_writes: "false",
          webhook_host: "localhost",
          webhook_port: "8787",
          dynamic_agent_creation_enabled: "true",
          dynamic_agent_creation_workspace_template: "employees/{sender_id}",
          dynamic_agent_creation_agent_dir_template: "agents/{sender_id}",
          dynamic_agent_creation_max_agents: "48",
        },
      });
    });

    expect(screen.getByText("飞书高级配置已保存")).toBeInTheDocument();
  });

  test("reflects official plugin runtime status returned by start command immediately", async () => {
    installInvokeMock({
      get_openclaw_plugin_feishu_runtime_status: async () => ({
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: false,
        started_at: null,
        last_stop_at: null,
        last_error: null,
        pid: null,
        port: null,
      }),
      start_openclaw_plugin_feishu_runtime: async () => ({
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: true,
        started_at: "2026-03-19T10:00:00Z",
        last_stop_at: null,
        last_error: null,
        pid: 43210,
        port: null,
      }),
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "保存官方插件配置" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "保存官方插件配置" }));

    await waitFor(() => {
      expect(screen.getByText("运行中")).toBeInTheDocument();
    });

    expect(screen.getByText("飞书官方插件运行中")).toBeInTheDocument();
  });

  test("shows official plugin runtime start errors returned by the start command", async () => {
    installInvokeMock({
      get_openclaw_plugin_feishu_runtime_status: async () => ({
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: false,
        started_at: null,
        last_stop_at: null,
        last_error: null,
        pid: null,
        port: null,
      }),
      start_openclaw_plugin_feishu_runtime: async () => ({
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: false,
        started_at: null,
        last_stop_at: "2026-03-19T10:00:10Z",
        last_error: "official feishu runtime exited with code 1",
        pid: null,
        port: null,
      }),
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "保存官方插件配置" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "保存官方插件配置" }));

    await waitFor(() => {
      expect(screen.getByText(/官方插件启动失败/)).toBeInTheDocument();
    });

    expect(screen.getByText(/official feishu runtime exited with code 1/)).toBeInTheDocument();
  });

  test("shows official runtime connection status when plugin runtime is running", async () => {
    installInvokeMock({
      get_openclaw_plugin_feishu_runtime_status: async () => ({
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: true,
        started_at: "2026-03-19T12:00:00Z",
        last_stop_at: null,
        last_error: null,
        pid: 4242,
        port: null,
      }),
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByText("运行中")).toBeInTheDocument();
    });

    expect(screen.getByText("飞书官方插件运行中")).toBeInTheDocument();
  });

  test("auto-starts official runtime on connector page load when plugin mode has saved credentials", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("start_openclaw_plugin_feishu_runtime", {
        pluginId: "openclaw-lark",
        accountId: null,
      });
    });
  });

  test("shows official-plugin diagnostics instead of legacy connector diagnostics in plugin mode", async () => {
    installInvokeMock({
      get_openclaw_plugin_feishu_runtime_status: async () => ({
        plugin_id: "openclaw-lark",
        account_id: "default",
        running: false,
        started_at: null,
        last_stop_at: null,
        last_event_at: "2026-03-19T12:00:01Z",
        last_error: null,
        pid: null,
        port: null,
        recent_logs: ["[info] channel/monitor: feishu[default]: WebSocket client started"],
      }),
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByText("配置模式")).toBeInTheDocument();
    });

    expect(screen.getByText("官方插件优先")).toBeInTheDocument();
    expect(screen.getAllByText("@larksuite/openclaw-lark").length).toBeGreaterThan(0);
    expect(screen.getByText("2026-03-19T12:00:01Z")).toBeInTheDocument();
    expect(screen.getByText(/\[info\] channel\/monitor: feishu\[default\]: WebSocket client started/)).toBeInTheDocument();
    expect(screen.queryByText("企业微信连接器")).not.toBeInTheDocument();
    expect(screen.queryByText("连接器诊断")).not.toBeInTheDocument();
  });

  test("denies feishu pairing requests from settings actions", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "配对与授权" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "配对与授权" }));
    fireEvent.click(screen.getByRole("button", { name: "拒绝" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("deny_feishu_pairing_request", {
        requestId: "pairing-1",
        resolvedByUser: "settings-ui",
      });
    });
  });

  test("filters pairing requests by status in the feishu console", async () => {
    installInvokeMock({
      list_feishu_pairing_requests: async () => [
        {
          id: "pairing-1",
          channel: "feishu",
          account_id: "default",
          sender_id: "ou_pending",
          chat_id: "ou_pending",
          code: "PAIR1234",
          status: "pending",
          created_at: "2026-03-19T10:00:00Z",
          updated_at: "2026-03-19T10:00:00Z",
          resolved_at: null,
          resolved_by_user: "",
        },
        {
          id: "pairing-2",
          channel: "feishu",
          account_id: "default",
          sender_id: "ou_approved",
          chat_id: "ou_approved",
          code: "PAIR5678",
          status: "approved",
          created_at: "2026-03-19T09:00:00Z",
          updated_at: "2026-03-19T09:10:00Z",
          resolved_at: "2026-03-19T09:10:00Z",
          resolved_by_user: "settings-ui",
        },
      ],
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "配对与授权" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "配对与授权" }));

    expect(screen.getByRole("button", { name: "待处理" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "已通过" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "全部" })).toBeInTheDocument();
    expect(screen.getByText("ou_pending · 待处理")).toBeInTheDocument();
    expect(screen.queryByText("ou_approved · 已通过")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "已通过" }));
    expect(screen.getByText("ou_approved · 已通过")).toBeInTheDocument();
    expect(screen.getByText(/申请时间 2026-03-19 09:00/)).toBeInTheDocument();
    expect(screen.getByText(/处理人 settings-ui · 处理时间 2026-03-19 09:10/)).toBeInTheDocument();
    expect(screen.queryByText("ou_pending · 待处理")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "全部" }));
    expect(screen.getByText("ou_pending · 待处理")).toBeInTheDocument();
    expect(screen.getByText("ou_approved · 已通过")).toBeInTheDocument();
  });

  test("shows guided empty states when official plugin host or pairing requests are missing", async () => {
    installInvokeMock({
      list_openclaw_plugin_channel_hosts: async () => [],
      list_feishu_pairing_requests: async () => [],
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "官方插件" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "官方插件" }));
    expect(screen.getByText("尚未识别到 OpenClaw 官方飞书插件")).toBeInTheDocument();
    expect(screen.getByText("@larksuite/openclaw-lark")).toBeInTheDocument();
    expect(screen.getByText(/账号视图和兼容宿主状态/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "安装官方插件" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "配对与授权" }));
    expect(screen.getByText("暂无待处理配对申请")).toBeInTheDocument();
    expect(screen.getByText(/新的飞书私聊配对请求会出现在这里/)).toBeInTheDocument();
  });

  test("installs the official feishu openclaw plugin from the empty state", async () => {
    let installed = false;
    installInvokeMock({
      list_openclaw_plugin_channel_hosts: async () => {
        if (!installed) {
          return [];
        }
        return [
          {
            plugin_id: "openclaw-lark",
            npm_spec: "@larksuite/openclaw-lark",
            version: "2026.3.17",
            channel: "feishu",
            display_name: "Feishu",
            capabilities: ["pairing"],
            reload_config_prefixes: ["channels.feishu"],
            target_hint: "<chatId|user:openId|chat:chatId>",
            docs_path: "/channels/feishu",
            status: "ready",
            error: null,
          },
        ];
      },
      install_openclaw_plugin_from_npm: async () => {
        installed = true;
        return {
          plugin_id: "openclaw-lark",
          npm_spec: "@larksuite/openclaw-lark",
          version: "2026.3.17",
          install_path: "D:/plugins/openclaw-lark",
          source_type: "npm",
          manifest_json: "{}",
          installed_at: "2026-03-19T10:00:00Z",
          updated_at: "2026-03-19T10:00:00Z",
        };
      },
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "安装官方插件" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "安装官方插件" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("install_openclaw_plugin_from_npm", {
        pluginId: "openclaw-lark",
        npmSpec: "@larksuite/openclaw-lark",
      });
    });

    await waitFor(() => {
      expect(screen.getAllByText("已识别").length).toBeGreaterThan(0);
    });
    expect(screen.getByText(/飞书官方插件已安装/)).toBeInTheDocument();
  });

  test("shows partial-load warnings when plugin host or pairing data fails to load", async () => {
    installInvokeMock({
      list_openclaw_plugin_channel_hosts: async () => {
        throw new Error("plugin host unavailable");
      },
      list_feishu_pairing_requests: async () => {
        throw new Error("pairing unavailable");
      },
    });

    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "渠道连接器" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "官方插件" })).toBeInTheDocument();
    });

    expect(screen.getAllByText("加载失败").length).toBeGreaterThan(0);

    fireEvent.click(screen.getByRole("button", { name: "官方插件" }));
    expect(screen.getByText("官方插件状态暂时不可用")).toBeInTheDocument();
    expect(screen.getByText(/请稍后刷新，或先检查插件宿主与安装状态/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "配对与授权" }));
    expect(screen.getByText("配对记录加载失败")).toBeInTheDocument();
    expect(screen.getByText(/暂时无法读取配对请求/)).toBeInTheDocument();
  });
});
