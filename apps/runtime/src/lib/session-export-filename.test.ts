import { describe, expect, test } from "vitest";
import { buildSessionExportFilename } from "./session-export-filename";

describe("buildSessionExportFilename", () => {
  test("uses display_title and local export timestamp in the default filename", () => {
    const fileName = buildSessionExportFilename(
      {
        title: "New Chat",
        display_title: "修复登录/接口:超时？",
      },
      new Date("2026-03-19T14:30:45"),
    );

    expect(fileName).toBe("修复登录-接口-超时-2026-03-19-1430.md");
  });

  test("falls back to a generic name when the title is empty after sanitizing", () => {
    const fileName = buildSessionExportFilename(
      {
        title: "  <>:\"/\\\\|?*  ",
      },
      new Date("2026-03-19T09:05:00"),
    );

    expect(fileName).toBe("session-export-2026-03-19-0905.md");
  });
});
