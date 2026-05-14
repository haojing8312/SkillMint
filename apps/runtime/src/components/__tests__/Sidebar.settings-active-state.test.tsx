import { render, screen } from "@testing-library/react";
import { Sidebar } from "../Sidebar";

function renderSidebar(options: { isSettingsActive?: boolean; collapsed?: boolean } = {}) {
  return render(
    <Sidebar
      activeMainView="start-task"
      isSettingsActive={options.isSettingsActive ?? false}
      onOpenStartTask={() => {}}
      onOpenExperts={() => {}}
      onOpenEmployees={() => {}}
      selectedSkillId="builtin-general"
      sessions={[]}
      selectedSessionId={null}
      onSelectSession={() => {}}
      onDeleteSession={() => {}}
      onSettings={() => {}}
      onSearchSessions={() => {}}
      onExportSession={() => {}}
      onCollapse={() => {}}
      collapsed={options.collapsed ?? false}
    />,
  );
}

describe("Sidebar settings active state", () => {
  test("makes settings the only selected navigation entry when settings is open", () => {
    renderSidebar({ isSettingsActive: true });

    expect(screen.getByRole("button", { name: "开始任务" })).toHaveAttribute("aria-pressed", "false");
    expect(screen.getByRole("button", { name: "开始任务" })).not.toHaveClass("bg-[var(--sm-primary-soft)]");
    expect(screen.getByRole("button", { name: "设置" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "设置" })).toHaveClass("bg-[var(--sm-primary-soft)]");
  });

  test("keeps the collapsed settings entry as the only selected navigation entry", () => {
    renderSidebar({ collapsed: true, isSettingsActive: true });

    expect(screen.getByRole("button", { name: "开始任务" })).toHaveAttribute("aria-pressed", "false");
    expect(screen.getByRole("button", { name: "设置" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "设置" })).toHaveClass("bg-[var(--sm-primary-soft)]");
  });
});
