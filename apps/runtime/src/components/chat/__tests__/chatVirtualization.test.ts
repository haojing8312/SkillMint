import { describe, expect, test } from "vitest";

import { computeVirtualWindow } from "../chatVirtualization";

describe("chat virtualization window", () => {
  test("renders only a bounded visible range for long sessions", () => {
    const heights = new Array(200).fill(80);

    const result = computeVirtualWindow({
      itemCount: 200,
      itemHeights: heights,
      scrollTop: 3200,
      viewportHeight: 640,
      overscan: 4,
      minVirtualizeCount: 40,
    });

    expect(result.virtualized).toBe(true);
    expect(result.startIndex).toBeGreaterThan(0);
    expect(result.endIndex).toBeLessThan(200);
    expect(result.endIndex - result.startIndex).toBeLessThan(40);
    expect(result.topSpacerHeight).toBe(2880);
    expect(result.bottomSpacerHeight).toBe(11840);
  });

  test("disables virtualization for short conversations", () => {
    const heights = new Array(12).fill(80);

    const result = computeVirtualWindow({
      itemCount: 12,
      itemHeights: heights,
      scrollTop: 0,
      viewportHeight: 640,
      overscan: 4,
      minVirtualizeCount: 40,
    });

    expect(result.virtualized).toBe(false);
    expect(result.startIndex).toBe(0);
    expect(result.endIndex).toBe(12);
    expect(result.topSpacerHeight).toBe(0);
    expect(result.bottomSpacerHeight).toBe(0);
  });
});
