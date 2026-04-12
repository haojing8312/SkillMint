import type {
  OpenClawLarkInstallerMode,
  OpenClawLarkInstallerSessionStatus,
} from "../../../types";

export function getLatestInstallerOutputLine(session: OpenClawLarkInstallerSessionStatus) {
  return session.recent_output.length > 0
    ? session.recent_output[session.recent_output.length - 1] ?? ""
    : "";
}

export function isFeishuInstallerFinished(session: OpenClawLarkInstallerSessionStatus) {
  return (
    !session.running &&
    !!session.mode &&
    session.recent_output.some((line) => line.includes("[system] official installer finished"))
  );
}

export function resolveFeishuInstallerCompletionNotice(session: OpenClawLarkInstallerSessionStatus) {
  if (!isFeishuInstallerFinished(session)) {
    return "";
  }

  const output = session.recent_output.join("\n");
  if (
    session.mode === "create" &&
    (output.includes("Success! Bot configured.") || output.includes("机器人配置成功"))
  ) {
    return "机器人创建已完成，请点击“启动连接”继续完成授权。";
  }

  if (session.mode === "link") {
    return "机器人关联已完成，请点击“启动连接”继续完成授权。";
  }

  return "安装向导已完成，请继续启动连接并完成授权。";
}

export function shouldShowFeishuInstallerGuidedPanel(
  branch: "existing_robot" | "create_robot" | null,
  session: OpenClawLarkInstallerSessionStatus,
) {
  return branch === "create_robot" && (session.running || session.recent_output.length > 0);
}

export function resolveFeishuInstallerFlowLabel(mode: OpenClawLarkInstallerMode | null) {
  if (mode === "create") {
    return "飞书官方创建机器人向导";
  }
  if (mode === "link") {
    return "飞书官方绑定机器人向导";
  }
  return "飞书官方向导";
}

export function looksLikeInstallerQrLine(line: string) {
  return /[█▀▄▌▐]/.test(line);
}

export function extractFeishuInstallerQrBlock(lines: string[]) {
  let bestStart = -1;
  let bestLength = 0;
  let currentStart = -1;
  let currentLength = 0;

  for (let index = 0; index < lines.length; index += 1) {
    if (looksLikeInstallerQrLine(lines[index] || "")) {
      if (currentStart === -1) {
        currentStart = index;
        currentLength = 0;
      }
      currentLength += 1;
      if (currentLength > bestLength) {
        bestStart = currentStart;
        bestLength = currentLength;
      }
    } else {
      currentStart = -1;
      currentLength = 0;
    }
  }

  if (bestStart === -1 || bestLength < 3) {
    return [];
  }
  return lines.slice(bestStart, bestStart + bestLength);
}

export function sanitizeFeishuInstallerDisplayLines(lines: string[]) {
  const qrBlock = extractFeishuInstallerQrBlock(lines);
  const qrSet = new Set(qrBlock);
  const filtered: string[] = [];
  let skipDebugObject = false;

  for (const rawLine of lines) {
    const line = rawLine ?? "";
    if (qrSet.has(line)) {
      continue;
    }
    if (line.startsWith("[DEBUG]") && line.includes("{")) {
      skipDebugObject = true;
      continue;
    }
    if (skipDebugObject) {
      if (line.trim() === "}") {
        skipDebugObject = false;
      }
      continue;
    }
    if (line.startsWith("[DEBUG]")) {
      continue;
    }
    filtered.push(line);
  }

  return filtered;
}
