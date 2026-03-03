import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { PackForm } from "../PackForm";

const invokeMock = vi.fn();
const saveMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: (...args: unknown[]) => saveMock(...args),
}));

describe("PackForm risk flow", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    saveMock.mockReset();
  });

  test("export action locks button while packing and shows completion message", async () => {
    let resolvePack: () => void = () => {};

    saveMock.mockResolvedValue("C:\\packs\\contract-helper.skillpack");
    invokeMock.mockImplementation((command: string) => {
      if (command === "pack_skill") {
        return new Promise<void>((resolve) => {
          resolvePack = resolve;
        });
      }
      return Promise.resolve(null);
    });

    render(
      <PackForm
        dirPath="C:\\skills\\contract-helper"
        frontMatter={{ name: "合同助手", description: "desc", version: "1.0.0", model: "gpt-4o-mini" }}
        fileCount={3}
      />
    );

    fireEvent.change(screen.getByPlaceholderText("例如：alice"), {
      target: { value: "alice" },
    });

    const exportButton = screen.getByRole("button", { name: "导出技能包" });
    fireEvent.click(exportButton);
    fireEvent.click(exportButton);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "打包中..." })).toBeDisabled();
    });

    const packCalls = invokeMock.mock.calls.filter(([command]) => command === "pack_skill");
    expect(packCalls).toHaveLength(1);

    resolvePack();

    await waitFor(() => {
      expect(screen.getByText("打包成功")).toBeInTheDocument();
    });
  });
});
