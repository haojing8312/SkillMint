import { fireEvent, render, screen } from "@testing-library/react";
import { Sidebar } from "../Sidebar";

describe("Sidebar risk flow", () => {
  test("switching to unrestricted requires confirmation", () => {
    const onChangeMode = vi.fn();

    render(
      <Sidebar
        activeMainView="start-task"
        onOpenStartTask={() => {}}
        onOpenExperts={() => {}}
        onOpenEmployees={() => {}}
        selectedSkillId="builtin-general"
        sessions={[]}
        selectedSessionId={null}
        onSelectSession={() => {}}
        newSessionPermissionMode="accept_edits"
        onChangeNewSessionPermissionMode={onChangeMode}
        onDeleteSession={() => {}}
        onSettings={() => {}}
        onSearchSessions={() => {}}
        onExportSession={() => {}}
        onCollapse={() => {}}
        collapsed={false}
      />
    );

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "unrestricted" } });
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("切换为全自动模式")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "取消" }));
    expect(onChangeMode).not.toHaveBeenCalled();

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "unrestricted" } });
    fireEvent.click(screen.getByRole("button", { name: "确认切换" }));
    expect(onChangeMode).toHaveBeenCalledTimes(1);
    expect(onChangeMode).toHaveBeenCalledWith("unrestricted");
  });
});
