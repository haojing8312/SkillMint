import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EmployeeHubView } from "../EmployeeHubView";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("EmployeeHubView employee creation flow", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === "get_runtime_preferences") {
        return Promise.resolve({ default_work_dir: "C:\\Users\\test\\SkillMint\\workspace" });
      }
      if (command === "set_runtime_preferences") return Promise.resolve(null);
      if (command === "resolve_default_work_dir") return Promise.resolve("C:\\Users\\test\\SkillMint\\workspace");
      return Promise.resolve(null);
    });
  });

  test("uses skill-first creation and hides manual employee form", async () => {
    const onOpenEmployeeCreatorSkill = vi.fn();

    render(
      <EmployeeHubView
        employees={[]}
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
        ]}
        selectedEmployeeId={null}
        onSelectEmployee={() => {}}
        onSaveEmployee={async () => {}}
        onDeleteEmployee={async () => {}}
        onSetAsMainAndEnter={() => {}}
        onStartTaskWithEmployee={() => {}}
        onOpenEmployeeCreatorSkill={onOpenEmployeeCreatorSkill}
      />
    );

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("get_runtime_preferences");
    });

    expect(screen.queryByRole("button", { name: "手动新建" })).not.toBeInTheDocument();
    expect(screen.queryByPlaceholderText("员工名称")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "保存员工" })).not.toBeInTheDocument();
    expect(screen.getByText("已移除手动创建流程，请通过「智能体员工助手」对话式完成创建与配置。")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "新建员工" }));
    expect(onOpenEmployeeCreatorSkill).toHaveBeenCalledTimes(1);
    expect(onOpenEmployeeCreatorSkill).toHaveBeenCalledWith({ mode: "create" });
  });
});
