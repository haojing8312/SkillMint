export type ComputeVirtualWindowArgs = {
  itemCount: number;
  itemHeights: number[];
  scrollTop: number;
  viewportHeight: number;
  overscan: number;
  minVirtualizeCount: number;
  forceIncludeIndex?: number | null;
};

export type VirtualWindow = {
  virtualized: boolean;
  startIndex: number;
  endIndex: number;
  topSpacerHeight: number;
  bottomSpacerHeight: number;
};

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function buildPrefixSums(itemHeights: number[]) {
  const prefix = new Array(itemHeights.length + 1).fill(0);
  for (let index = 0; index < itemHeights.length; index += 1) {
    prefix[index + 1] = prefix[index] + Math.max(0, itemHeights[index] || 0);
  }
  return prefix;
}

function findStartIndex(prefixSums: number[], scrollTop: number) {
  let low = 0;
  let high = Math.max(0, prefixSums.length - 2);
  while (low < high) {
    const mid = Math.floor((low + high) / 2);
    if (prefixSums[mid + 1] > scrollTop) {
      high = mid;
    } else {
      low = mid + 1;
    }
  }
  return low;
}

function findEndIndex(prefixSums: number[], viewportBottom: number, itemCount: number) {
  let low = 0;
  let high = itemCount;
  while (low < high) {
    const mid = Math.floor((low + high) / 2);
    if (prefixSums[mid] >= viewportBottom) {
      high = mid;
    } else {
      low = mid + 1;
    }
  }
  return clamp(low, 0, itemCount);
}

export function computeVirtualWindow({
  itemCount,
  itemHeights,
  scrollTop,
  viewportHeight,
  overscan,
  minVirtualizeCount,
  forceIncludeIndex = null,
}: ComputeVirtualWindowArgs): VirtualWindow {
  if (itemCount <= 0) {
    return {
      virtualized: false,
      startIndex: 0,
      endIndex: 0,
      topSpacerHeight: 0,
      bottomSpacerHeight: 0,
    };
  }

  if (itemCount < minVirtualizeCount || viewportHeight <= 0) {
    return {
      virtualized: false,
      startIndex: 0,
      endIndex: itemCount,
      topSpacerHeight: 0,
      bottomSpacerHeight: 0,
    };
  }

  const normalizedHeights =
    itemHeights.length === itemCount
      ? itemHeights
      : new Array(itemCount).fill(0).map((_, index) => itemHeights[index] || 0);
  const prefixSums = buildPrefixSums(normalizedHeights);
  const viewportStart = Math.max(0, scrollTop);
  const viewportBottom = viewportStart + Math.max(1, viewportHeight);

  let startIndex = Math.max(0, findStartIndex(prefixSums, viewportStart) - overscan);
  let endIndex = Math.min(itemCount, findEndIndex(prefixSums, viewportBottom, itemCount) + overscan);

  if (endIndex <= startIndex) {
    endIndex = Math.min(itemCount, startIndex + 1);
  }

  if (typeof forceIncludeIndex === "number" && forceIncludeIndex >= 0 && forceIncludeIndex < itemCount) {
    if (forceIncludeIndex < startIndex) {
      const windowSize = endIndex - startIndex;
      startIndex = forceIncludeIndex;
      endIndex = Math.min(itemCount, startIndex + windowSize);
    } else if (forceIncludeIndex >= endIndex) {
      const windowSize = endIndex - startIndex;
      endIndex = forceIncludeIndex + 1;
      startIndex = Math.max(0, endIndex - windowSize);
    }
  }

  return {
    virtualized: true,
    startIndex,
    endIndex,
    topSpacerHeight: prefixSums[startIndex],
    bottomSpacerHeight: prefixSums[itemCount] - prefixSums[endIndex],
  };
}
