import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { InstallDialog } from "../InstallDialog";

const invokeMock = vi.fn();
const openMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: (...args: unknown[]) => openMock(...args),
}));

describe("InstallDialog marketplace mode", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    openMock.mockReset();
  });

  test("shows marketplace copy and installs a searched skill", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "search_clawhub_skills") {
        return Promise.resolve([
          {
            slug: "ai-recruiting-engine",
            name: "AI Recruiting Engine",
            description: "Automates recruiting workflows",
            stars: 42,
            github_url: "https://github.com/example/ai-recruiting-engine",
          },
        ]);
      }
      if (command === "install_clawhub_skill") {
        return Promise.resolve({
          manifest: { id: "clawhub-ai-recruiting-engine" },
          missing_mcp: [],
        });
      }
      return Promise.resolve(null);
    });

    const onInstalled = vi.fn();
    const onClose = vi.fn();
    render(<InstallDialog onInstalled={onInstalled} onClose={onClose} />);

    fireEvent.click(screen.getByRole("button", { name: "技能库" }));
    expect(
      screen.getByPlaceholderText("输入关键词搜索 SkillHub / ClawHub 技能"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("通过关键字搜索 SkillHub / ClawHub 公共技能后可直接安装。"),
    ).toBeInTheDocument();

    fireEvent.change(screen.getByPlaceholderText("输入关键词搜索 SkillHub / ClawHub 技能"), {
      target: { value: "AI Recruiting Engine" },
    });
    fireEvent.click(screen.getByRole("button", { name: "搜索" }));

    await waitFor(() => {
      expect(screen.getByText("AI Recruiting Engine")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("AI Recruiting Engine"));
    fireEvent.click(screen.getByRole("button", { name: "安装" }));

    await waitFor(() => {
      expect(screen.getByText(/确认安装「AI Recruiting Engine」吗/)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "确认安装" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("search_clawhub_skills", {
        query: "AI Recruiting Engine",
        page: 1,
        limit: 20,
      });
      expect(invokeMock).toHaveBeenCalledWith("install_clawhub_skill", {
        slug: "ai-recruiting-engine",
        githubUrl: "https://github.com/example/ai-recruiting-engine",
      });
      expect(onInstalled).toHaveBeenCalledWith("clawhub-ai-recruiting-engine");
      expect(onClose).toHaveBeenCalled();
    });
  });
});
