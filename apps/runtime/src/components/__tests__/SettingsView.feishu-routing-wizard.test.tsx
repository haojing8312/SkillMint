import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SettingsView } from "../SettingsView";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("SettingsView feishu routing wizard", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === "list_model_configs") return Promise.resolve([]);
      if (command === "list_mcp_servers") return Promise.resolve([]);
      if (command === "list_search_configs") return Promise.resolve([]);
      if (command === "get_routing_settings") {
        return Promise.resolve({ max_call_depth: 4, node_timeout_seconds: 60, retry_count: 0 });
      }
      if (command === "list_builtin_provider_plugins") return Promise.resolve([]);
      if (command === "list_provider_configs") return Promise.resolve([]);
      if (command === "get_capability_routing_policy") return Promise.resolve(null);
      if (command === "list_capability_route_templates") return Promise.resolve([]);
      if (command === "get_feishu_gateway_settings") {
        return Promise.resolve({
          app_id: "",
          app_secret: "",
          ingress_token: "",
          encrypt_key: "",
          sidecar_base_url: "http://localhost:8765",
        });
      }
      if (command === "get_feishu_long_connection_status") {
        return Promise.resolve({ running: false, started_at: null, queued_events: 0 });
      }
      if (command === "get_feishu_event_relay_status") {
        return Promise.resolve({
          running: false,
          generation: 0,
          interval_ms: 1500,
          total_accepted: 0,
          last_error: null,
        });
      }
      if (command === "list_feishu_chats") {
        return Promise.resolve({ items: [], has_more: false, page_token: "" });
      }
      if (command === "list_recent_im_threads") return Promise.resolve([]);
      if (command === "list_agent_employees") return Promise.resolve([]);
      if (command === "list_im_routing_bindings") return Promise.resolve([]);
      if (command === "list_skills") return Promise.resolve([]);
      if (command === "get_runtime_preferences") {
        return Promise.resolve({ default_work_dir: "" });
      }
      if (command === "upsert_im_routing_binding") return Promise.resolve("rule-1");
      if (command === "simulate_im_route") {
        return Promise.resolve({ agentId: "main", matchedBy: "default" });
      }
      return Promise.resolve(null);
    });
  });

  test("saves routing rule from wizard and can run simulation", async () => {
    render(<SettingsView onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "飞书协作" }));

    await waitFor(() => {
      expect(screen.getByText("飞书路由规则向导")).toBeInTheDocument();
    });

    fireEvent.change(screen.getByPlaceholderText("agent_id（如 main）"), {
      target: { value: "peer-agent" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存规则" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "upsert_im_routing_binding",
        expect.objectContaining({
          input: expect.objectContaining({ agent_id: "peer-agent" }),
        }),
      );
    });

    fireEvent.click(screen.getByRole("button", { name: "模拟路由" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "simulate_im_route",
        expect.objectContaining({
          payload: expect.objectContaining({ channel: "feishu" }),
        }),
      );
    });
  });
});
