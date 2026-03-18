import { fireEvent, render, screen } from "@testing-library/react";
import { ToolIsland } from "../ToolIsland";

describe("ToolIsland", () => {
  test("uses user-facing summaries instead of raw engineering wording", () => {
    render(
      <ToolIsland
        isRunning={false}
        toolCalls={[
          {
            id: "search-1",
            name: "web_search",
            input: { query: "US military presence Middle East 2025" },
            output: "done",
            status: "completed",
          },
          {
            id: "write-1",
            name: "write_file",
            input: { path: "conflict_report.html" },
            output: "done",
            status: "completed",
          },
        ]}
      />,
    );

    expect(screen.getByText("执行记录")).toBeInTheDocument();
    expect(screen.getByText("2 个步骤")).toBeInTheDocument();
    expect(screen.queryByText("已执行 2 个操作")).not.toBeInTheDocument();

    fireEvent.click(screen.getByTestId("tool-island-summary"));

    expect(screen.getByText("网页搜索")).toBeInTheDocument();
    expect(screen.getByText("写入文件")).toBeInTheDocument();
    expect(screen.queryByText("web_search")).not.toBeInTheDocument();
    expect(screen.queryByText("write_file")).not.toBeInTheDocument();
  });

  test("renders structured tool summaries instead of raw json blobs", () => {
    render(
      <ToolIsland
        isRunning={false}
        toolCalls={[
          {
            id: "write-structured",
            name: "write_file",
            input: { path: "structured-report.html" },
            output: JSON.stringify({
              ok: true,
              tool: "write_file",
              summary: "成功写入 12 字节到 structured-report.html",
              details: {
                path: "structured-report.html",
                absolute_path: "E:/workspace/structured-report.html",
                bytes_written: 12,
              },
            }),
            status: "completed",
          },
        ]}
      />,
    );

    fireEvent.click(screen.getByText("执行记录"));
    fireEvent.click(screen.getByTestId("tool-island-step-write-structured"));

    expect(screen.getByText("成功写入 12 字节到 structured-report.html")).toBeInTheDocument();
    expect(screen.queryByText(/"summary"/)).not.toBeInTheDocument();
    expect(screen.getByText(/"bytes_written": 12/)).toBeInTheDocument();
  });
});
