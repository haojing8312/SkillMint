import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EmployeeHubView } from "../EmployeeHubView";

const invokeMock = vi.fn();
const saveMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: (...args: unknown[]) => saveMock(...args),
}));

describe("EmployeeHubView memory governance", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    saveMock.mockReset();
    saveMock.mockResolvedValue("D:\\exports\\sales-memory.json");
    let skillSalesLifecycleState = "active";
    invokeMock.mockImplementation((command: string, payload?: any) => {
      if (command === "get_runtime_preferences") {
        return Promise.resolve({ default_work_dir: "C:\\Users\\test\\.workclaw\\workspace" });
      }
      if (command === "set_runtime_preferences") return Promise.resolve(null);
      if (command === "resolve_default_work_dir") {
        return Promise.resolve("C:\\Users\\test\\.workclaw\\workspace");
      }
      if (command === "list_im_channel_registry") {
        return Promise.resolve([
          {
            channel: "feishu",
            runtime_status: {
              plugin_id: "@larksuite/openclaw-lark",
              account_id: "default",
              running: true,
              started_at: "2026-03-04T00:00:00Z",
              last_error: null,
              last_event_at: null,
              recent_logs: [],
            },
          },
        ]);
      }
      if (command === "get_feishu_employee_connection_statuses") {
        return Promise.resolve({
          relay: { running: true, generation: 1, interval_ms: 1500, total_accepted: 0, last_error: null },
          sidecar: { running: true, started_at: null, queued_events: 0, running_count: 0, items: [] },
        });
      }
      if (command === "get_employee_profile_memory_status") {
        return Promise.resolve({
          employee_id: "sales_lead",
          profile_id: "profile-sales",
          skill_id: payload?.skillId || "skill-sales",
          profile_memory_dir: "D:/WorkClaw/profiles/profile-sales/memories",
          profile_memory_file_path: "D:/WorkClaw/profiles/profile-sales/memories/MEMORY.md",
          profile_memory_file_exists: true,
          active_source: "profile",
          active_source_path: "D:/WorkClaw/profiles/profile-sales/memories/MEMORY.md",
        });
      }
      if (command === "get_agent_profile_files") {
        return Promise.resolve({
          employee_id: "sales_lead",
          employee_name: "销售主管",
          profile_dir: "D:/WorkClaw/profiles/profile-sales",
          artifacts: [
            {
              name: "instructions",
              path: "D:/WorkClaw/profiles/profile-sales/instructions",
              exists: true,
              file_count: 3,
            },
            {
              name: "memories",
              path: "D:/WorkClaw/profiles/profile-sales/memories",
              exists: true,
              file_count: 2,
            },
            {
              name: "curator",
              path: "D:/WorkClaw/profiles/profile-sales/curator",
              exists: true,
              file_count: 1,
            },
          ],
          files: [
            {
              name: "RULES.md",
              path: "D:/WorkClaw/profiles/profile-sales/instructions/RULES.md",
              exists: true,
              content: "# RULES\n\n销售主管 profile rules",
              error: null,
            },
            {
              name: "PERSONA.md",
              path: "D:/WorkClaw/profiles/profile-sales/instructions/PERSONA.md",
              exists: true,
              content: "# PERSONA\n\n专业直接",
              error: null,
            },
            {
              name: "USER_CONTEXT.md",
              path: "D:/WorkClaw/profiles/profile-sales/instructions/USER_CONTEXT.md",
              exists: true,
              content: "# USER_CONTEXT\n\n先给结论",
              error: null,
            },
          ],
        });
      }
      if (command === "list_employee_growth_events") {
        return Promise.resolve({
          employee_id: "emp-sales",
          profile_id: "profile-sales",
          events: [
            {
              id: "growth-1",
              profile_id: "profile-sales",
              session_id: "session-growth",
              event_type: "skill_patch",
              target_type: "skill",
              target_id: "skill-sales",
              summary: "沉淀销售周报流程",
              evidence_json: { version_id: "skv-1" },
              created_at: "2026-05-08T01:00:00Z",
            },
            {
              id: "growth-2",
              profile_id: "profile-sales",
              session_id: "session-memory",
              event_type: "memory_add",
              target_type: "profile_memory",
              target_id: "MEMORY",
              summary: "记住客户偏好先给结论",
              evidence_json: { version_id: "v20260508T010101000000Z" },
              created_at: "2026-05-08T01:01:00Z",
            },
            {
              id: "growth-3",
              profile_id: "profile-sales",
              session_id: "session-correction",
              event_type: "user_correction",
              target_type: "profile_memory",
              target_id: "MEMORY",
              summary: "纠正日报摘要格式",
              evidence_json: { version_id: "v20260508T010102000000Z" },
              created_at: "2026-05-08T01:02:00Z",
            },
          ],
        });
      }
      if (command === "list_employee_curator_runs") {
        return Promise.resolve({
          employee_id: "emp-sales",
          profile_id: "profile-sales",
          runs: [
            {
              id: "cur-1",
              profile_id: "profile-sales",
              scope: "profile",
              summary: "发现 2 个可整理项",
              report_path: "D:/WorkClaw/profiles/profile-sales/curator/reports/cur-1.json",
              mode: "run",
              has_state_changes: true,
              changed_targets: [
                {
                  kind: "stale_skill",
                  target_type: "skill",
                  target_id: "skill-sales",
                  state_changed: true,
                  restored_to: "",
                  suggested_action: "curator.restore",
                  reversible: true,
                },
              ],
              restore_candidates: [
                {
                  target_type: "skill",
                  target_id: "skill-sales",
                  tool: "curator",
                  action: "restore",
                  input: { action: "restore", skill_id: "skill-sales" },
                },
              ],
              created_at: "2026-05-08T02:00:00Z",
              findings: [
                {
                  kind: "duplicate_memory",
                  severity: "medium",
                  target_type: "memory",
                  target_id: "MEMORY.md",
                  summary: "Profile Memory 中存在重复条目",
                  evidence_json: { line: 2 },
                  suggested_action: "memory.replace",
                  reversible: true,
                },
              ],
            },
          ],
        });
      }
      if (command === "restore_employee_curator_stale_skill") {
        return Promise.resolve({
          id: "cur-restore",
          profile_id: "profile-sales",
          scope: "profile",
          summary: "已将 stale skill 恢复为 active: skill-sales",
          report_path: "D:/WorkClaw/profiles/profile-sales/curator/reports/cur-restore.json",
          mode: "restore",
          has_state_changes: true,
          changed_targets: [
            {
              kind: "curator_restore",
              target_type: "skill",
              target_id: payload?.skillId,
              state_changed: true,
              restored_to: "active",
              suggested_action: "继续观察该技能",
              reversible: true,
            },
          ],
          restore_candidates: [],
          findings: [],
          created_at: "2026-05-08T02:01:00Z",
        });
      }
      if (command === "list_skill_os_index") {
        return Promise.resolve([
          {
            skill_id: "skill-sales",
            name: "销售助手",
            description: "销售流程",
            version: "1.0.0",
            tags: ["sales"],
            source: {
              raw_source_type: "preset",
              canonical: "preset",
              immutable_content: false,
              directory_backed: true,
              requires_unpack_for_view: false,
            },
            capabilities: {
              can_list: true,
              can_view: true,
              can_patch: true,
              can_archive: true,
              can_reset: true,
              can_agent_delete: true,
              can_user_uninstall: true,
            },
            lifecycle_state: skillSalesLifecycleState,
            usage: {
              view_count: 1,
              use_count: 0,
              patch_count: 1,
              last_viewed_at: "2026-05-08T01:00:00Z",
              last_used_at: "",
              last_patched_at: "2026-05-08T01:00:00Z",
              pinned: false,
            },
            toolset_policy: {
              requires_toolsets: ["memory", "skills"],
              optional_toolsets: ["web"],
              denied_toolsets: [],
              unknown_toolsets: [],
            },
          },
          {
            skill_id: "skill-support",
            name: "客服知识库",
            description: "客服流程",
            version: "1.0.0",
            tags: ["support"],
            source: {
              raw_source_type: "encrypted",
              canonical: "skillpack",
              immutable_content: true,
              directory_backed: false,
              requires_unpack_for_view: true,
            },
            capabilities: {
              can_list: true,
              can_view: true,
              can_patch: false,
              can_archive: false,
              can_reset: false,
              can_agent_delete: false,
              can_user_uninstall: true,
            },
            lifecycle_state: payload?.skillId === "skill-sales" ? skillSalesLifecycleState : "active",
            usage: {
              view_count: 0,
              use_count: 0,
              patch_count: 0,
              last_viewed_at: "",
              last_used_at: "",
              last_patched_at: "",
              pinned: false,
            },
            toolset_policy: {
              requires_toolsets: [],
              optional_toolsets: [],
              denied_toolsets: [],
              unknown_toolsets: [],
            },
          },
        ]);
      }
      if (command === "get_skill_os_view") {
        return Promise.resolve({
          entry: {
            skill_id: payload?.skillId,
            name: payload?.skillId === "skill-sales" ? "销售助手" : "客服知识库",
            description: "",
            version: "1.0.0",
            tags: [],
            source: {
              raw_source_type: payload?.skillId === "skill-sales" ? "preset" : "encrypted",
              canonical: payload?.skillId === "skill-sales" ? "preset" : "skillpack",
              immutable_content: payload?.skillId !== "skill-sales",
              directory_backed: payload?.skillId === "skill-sales",
              requires_unpack_for_view: payload?.skillId !== "skill-sales",
            },
            capabilities: {
              can_list: true,
              can_view: true,
              can_patch: payload?.skillId === "skill-sales",
              can_archive: payload?.skillId === "skill-sales",
              can_reset: payload?.skillId === "skill-sales",
              can_agent_delete: payload?.skillId === "skill-sales",
              can_user_uninstall: true,
            },
            lifecycle_state: payload?.skillId === "skill-sales" ? skillSalesLifecycleState : "active",
            usage: {
              view_count: 1,
              use_count: 0,
              patch_count: payload?.skillId === "skill-sales" ? 1 : 0,
              last_viewed_at: "2026-05-08T01:00:00Z",
              last_used_at: "",
              last_patched_at: payload?.skillId === "skill-sales" ? "2026-05-08T01:00:00Z" : "",
              pinned: false,
            },
            toolset_policy: {
              requires_toolsets: payload?.skillId === "skill-sales" ? ["memory", "skills"] : [],
              optional_toolsets: payload?.skillId === "skill-sales" ? ["web"] : [],
              denied_toolsets: [],
              unknown_toolsets: [],
            },
          },
          content: "# 销售助手\n\n沉淀销售周报流程。",
          read_only: payload?.skillId !== "skill-sales",
          derived: false,
        });
      }
      if (command === "list_skill_os_versions") {
        return Promise.resolve([
          {
            version_id: "skv-1",
            skill_id: payload?.skillId,
            source_type: "preset",
            action: "patch",
            summary: "沉淀销售周报流程",
            created_at: "2026-05-08T01:00:00Z",
          },
        ]);
      }
      if (command === "patch_skill_os") {
        return Promise.resolve({
          action: "skill_patch",
          skill: {},
          version_id: "skv-patch",
          growth_event_id: "growth-patch",
          diff: "-沉淀销售周报流程。\n+沉淀销售周报流程，输出先给结论。",
        });
      }
      if (command === "pin_skill_os") return Promise.resolve(null);
      if (command === "reset_skill_os") {
        return Promise.resolve({
          action: "skill_reset",
          skill: {},
          version_id: "skv-reset",
          reset_to_version_id: "skv-baseline",
          growth_event_id: "growth-reset",
          diff: "-Changed body\n+Baseline body",
        });
      }
      if (command === "rollback_skill_os") {
        return Promise.resolve({
          action: "skill_rollback",
          skill: {},
          version_id: "skv-rollback",
          rollback_to_version_id: payload?.versionId,
          growth_event_id: "growth-rollback",
          diff: "-Current body\n+Old body",
        });
      }
      if (command === "archive_skill_os") {
        skillSalesLifecycleState = "archived";
        return Promise.resolve({
          action: "skill_archive",
          skill: {},
          version_id: "skv-archive",
          growth_event_id: "growth-archive",
          diff: "",
        });
      }
      if (command === "restore_skill_os") {
        skillSalesLifecycleState = "active";
        return Promise.resolve({
          action: "skill_restore",
          skill: {},
          version_id: "skv-restore",
          growth_event_id: "growth-restore",
          diff: "",
        });
      }
      if (command === "delete_skill_os") {
        return Promise.resolve({
          action: "skill_delete",
          skill: {},
          version_id: "skv-delete",
          growth_event_id: "growth-delete",
          diff: "",
        });
      }
      if (command === "export_agent_profile") {
        return Promise.resolve({
          employee_id: "sales_lead",
          employee_name: "销售主管",
          profile_id: "profile-sales",
          profile_dir: "D:/WorkClaw/profiles/profile-sales",
          export_path: payload?.outputPath,
          file_count: 8,
          total_bytes: 512,
        });
      }
      if (command === "write_export_file") return Promise.resolve(null);
      return Promise.resolve(null);
    });
  });

  test("supports refresh export and clear for selected employee memory", async () => {
    render(
      <EmployeeHubView
        employees={[
          {
            id: "emp-sales",
            employee_id: "sales_lead",
            name: "销售主管",
            role_id: "sales_lead",
            persona: "",
            feishu_open_id: "",
            feishu_app_id: "",
            feishu_app_secret: "",
            primary_skill_id: "skill-sales",
            default_work_dir: "",
            openclaw_agent_id: "sales_lead",
            routing_priority: 100,
            enabled_scopes: ["feishu"],
            enabled: true,
            is_default: false,
            skill_ids: ["skill-sales", "skill-support"],
            created_at: "2026-03-01T00:00:00Z",
            updated_at: "2026-03-01T00:00:00Z",
          },
        ]}
        skills={[
          {
            id: "builtin-general",
            name: "通用助手",
            description: "",
            version: "1.0.0",
            author: "",
            recommended_model: "",
            tags: [],
            created_at: "2026-03-01T00:00:00Z",
          },
          {
            id: "skill-sales",
            name: "销售助手",
            description: "",
            version: "1.0.0",
            author: "",
            recommended_model: "",
            tags: [],
            created_at: "2026-03-01T00:00:00Z",
          },
        ]}
        selectedEmployeeId="emp-sales"
        onSelectEmployee={() => {}}
        onSaveEmployee={async () => {}}
        onDeleteEmployee={async () => {}}
        onSetAsMainAndEnter={() => {}}
        onStartTaskWithEmployee={() => {}}
      />,
    );

    expect(invokeMock).not.toHaveBeenCalledWith("get_employee_memory_stats", expect.anything());
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("get_employee_profile_memory_status", {
        employeeId: "sales_lead",
        skillId: "skill-sales",
        profileId: null,
        workDir: null,
        imRoleId: null,
      });
    });
    expect(screen.getByTestId("employee-profile-memory-source")).toHaveTextContent("Profile Home");
    expect(screen.getByTestId("employee-profile-memory-skill")).toHaveTextContent("skill-sales");
    expect(screen.queryByTestId("employee-profile-memory-legacy")).not.toBeInTheDocument();
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_employee_growth_events", {
        employeeId: "emp-sales",
        limit: 12,
      });
    });
    expect(screen.getByTestId("employee-growth-profile")).toHaveTextContent("Profile Memory OS");
    expect(screen.getByTestId("employee-growth-profile")).toHaveAttribute("title", "profile-sales");
    expect(screen.getByTestId("employee-profile-artifacts")).toHaveTextContent("instructions");
    expect(screen.getByTestId("employee-profile-artifacts")).toHaveTextContent("memories");
    expect(screen.getByTestId("employee-profile-artifacts")).toHaveTextContent("curator");
    expect(screen.getByTestId("employee-profile-runtime-status")).toHaveTextContent("AI 员工运行时状态");
    expect(screen.getByTestId("employee-profile-runtime-status")).toHaveTextContent("profile-sales");
    expect(screen.getByTestId("employee-profile-runtime-memory")).toHaveTextContent("Profile Memory 可用");
    expect(screen.getByTestId("employee-profile-runtime-skills")).toHaveTextContent("2 个技能授权");
    expect(screen.getByTestId("employee-profile-runtime-toolsets")).toHaveTextContent("memory · web");
    expect(screen.getByTestId("employee-profile-runtime-growth")).toHaveTextContent("3 条成长证据");
    expect(screen.getByTestId("employee-profile-runtime-curator")).toHaveTextContent("1 份 Curator 报告");
    expect(screen.getByTestId("employee-profile-runtime-status")).toHaveTextContent("Canonical Profile Runtime");
    expect(screen.getByText("导出 Profile")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("employee-profile-export"));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("export_agent_profile", {
        employeeDbId: "emp-sales",
        outputPath: "D:\\exports\\sales-memory.json",
      });
    });
    expect(screen.getByTestId("employee-growth-events")).toHaveTextContent("优化技能");
    expect(screen.getByTestId("employee-growth-events")).toHaveTextContent("沉淀销售周报流程");
    expect(screen.getByTestId("employee-growth-events")).toHaveTextContent("写入记忆");
    expect(screen.getByTestId("employee-growth-events")).toHaveTextContent("记住客户偏好先给结论");
    expect(screen.getByTestId("employee-growth-events")).toHaveTextContent("用户纠正");
    expect(screen.getByTestId("employee-growth-events")).toHaveTextContent("纠正日报摘要格式");
    expect(screen.queryByTestId("employee-growth-export")).not.toBeInTheDocument();
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_employee_curator_runs", {
        employeeId: "emp-sales",
        limit: 5,
      });
    });
    expect(screen.getByTestId("employee-curator-report")).toHaveTextContent("发现 2 个可整理项");
    expect(screen.getByTestId("employee-curator-report")).toHaveTextContent("Run");
    expect(screen.getByTestId("employee-curator-report")).toHaveTextContent("已变更");
    expect(screen.getByTestId("employee-curator-report")).toHaveTextContent("恢复 skill-sales");
    expect(screen.getByTestId("employee-curator-report")).toHaveTextContent("重复记忆");
    expect(screen.getByTestId("employee-curator-report")).toHaveTextContent("memory.replace");
    fireEvent.click(screen.getByTestId("employee-curator-toggle-report"));
    expect(screen.getByTestId("employee-curator-report-detail")).toHaveTextContent("Report JSON");
    expect(screen.getByTestId("employee-curator-report-detail")).toHaveTextContent("stale_skill");
    expect(screen.getByTestId("employee-curator-report-detail")).toHaveTextContent("restore_candidates");
    fireEvent.click(screen.getByTestId("employee-curator-restore-skill-sales"));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("restore_employee_curator_stale_skill", {
        employeeId: "emp-sales",
        skillId: "skill-sales",
      });
    });
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_skill_os_index");
    });
    expect(screen.getByTestId("employee-skill-os")).toHaveTextContent("销售助手");
    expect(screen.getByTestId("employee-skill-os")).toHaveTextContent("Preset · 可进化");
    expect(screen.getByTestId("employee-skill-os")).toHaveTextContent(".skillpack · 只读");
    expect(screen.getByTestId("employee-skill-os-toolsets")).toHaveTextContent("requires:memory");
    expect(screen.getByTestId("employee-skill-os-usage")).toHaveTextContent("view 1");
    fireEvent.click(screen.getByTestId("employee-skill-os-pin"));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("pin_skill_os", {
        skillId: "skill-sales",
        pinned: true,
      });
    });
    expect(screen.getByTestId("employee-skill-os-versions")).toHaveTextContent("沉淀销售周报流程");
    fireEvent.click(screen.getByTestId("employee-skill-os-edit"));
    const editor = screen.getByTestId("employee-skill-os-editor").querySelector("textarea");
    expect(editor).not.toBeNull();
    fireEvent.change(editor as HTMLTextAreaElement, {
      target: { value: "# 销售助手\n\n沉淀销售周报流程，输出先给结论。" },
    });
    expect(screen.getByTestId("employee-skill-os-diff")).toHaveTextContent("+沉淀销售周报流程，输出先给结论。");
    fireEvent.click(screen.getByTestId("employee-skill-os-save-patch"));
    expect(screen.getByRole("dialog")).toHaveTextContent("更新技能");
    fireEvent.click(screen.getByRole("button", { name: "确认执行" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("patch_skill_os", {
        skillId: "skill-sales",
        content: "# 销售助手\n\n沉淀销售周报流程，输出先给结论。",
        employeeId: "emp-sales",
        summary: "Patch skill from employee workbench",
        confirm: true,
      });
    });
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("employee-skill-os-reset"));
    expect(screen.getByRole("dialog")).toHaveTextContent("重置技能");
    fireEvent.click(screen.getByRole("button", { name: "确认执行" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("reset_skill_os", {
        skillId: "skill-sales",
        employeeId: "emp-sales",
        summary: "Reset skill from employee workbench",
        confirm: true,
      });
    });
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("employee-skill-os-rollback-skv-1"));
    expect(screen.getByRole("dialog")).toHaveTextContent("回滚技能");
    fireEvent.click(screen.getByRole("button", { name: "确认执行" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("rollback_skill_os", {
        skillId: "skill-sales",
        versionId: "skv-1",
        employeeId: "emp-sales",
        summary: "Rollback skill from employee workbench",
        confirm: true,
      });
    });
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("employee-skill-os-archive"));
    expect(screen.getByRole("dialog")).toHaveTextContent("归档技能");
    fireEvent.click(screen.getByRole("button", { name: "确认执行" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("archive_skill_os", {
        skillId: "skill-sales",
        employeeId: "emp-sales",
        summary: "Archive skill from employee workbench",
        confirm: true,
      });
    });
    await waitFor(() => {
      expect(screen.getByTestId("employee-skill-os-detail")).toHaveTextContent("archived");
      expect(screen.getByTestId("employee-skill-os-restore")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("employee-skill-os-restore"));
    expect(screen.getByRole("dialog")).toHaveTextContent("恢复技能");
    fireEvent.click(screen.getByRole("button", { name: "确认执行" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("restore_skill_os", {
        skillId: "skill-sales",
        employeeId: "emp-sales",
        summary: "Restore skill from employee workbench",
      });
    });
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("employee-skill-os-delete"));
    expect(screen.getByRole("dialog")).toHaveTextContent("删除技能");
    expect(screen.getByRole("dialog")).toHaveTextContent("该操作不可逆");
    fireEvent.click(screen.getByRole("button", { name: "确认执行" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("delete_skill_os", {
        skillId: "skill-sales",
        employeeId: "emp-sales",
        summary: "Delete skill from employee workbench",
        confirm: true,
      });
    });
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
    expect(invokeMock).not.toHaveBeenCalledWith("list_employee_growth_reviews", expect.anything());

    expect(screen.queryByTestId("employee-memory-export")).not.toBeInTheDocument();
    expect(screen.queryByTestId("employee-memory-clear")).not.toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalledWith("export_employee_memory", expect.anything());
    expect(invokeMock).not.toHaveBeenCalledWith("clear_employee_memory", expect.anything());
  });
});
