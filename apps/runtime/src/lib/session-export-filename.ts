import type { SessionInfo } from "../types";

const WINDOWS_RESERVED_RE = /[<>:"/\\|?*\u0000-\u001f\uff1c\uff1e\uff1a\uff02\uff0f\uff3c\uff5c\uff1f\uff0a]/g;
const WHITESPACE_RE = /\s+/g;
const DASH_RUN_RE = /-+/g;

function sanitizeSegment(value: string): string {
  return value
    .trim()
    .replace(WINDOWS_RESERVED_RE, "-")
    .replace(WHITESPACE_RE, "-")
    .replace(DASH_RUN_RE, "-")
    .replace(/^[.\- ]+|[.\- ]+$/g, "");
}

function pad2(value: number): string {
  return String(value).padStart(2, "0");
}

function formatTimestamp(date: Date): string {
  return [
    date.getFullYear(),
    pad2(date.getMonth() + 1),
    pad2(date.getDate()),
  ].join("-") + `-${pad2(date.getHours())}${pad2(date.getMinutes())}`;
}

export function buildSessionExportFilename(session?: Pick<SessionInfo, "title" | "display_title"> | null, now = new Date()): string {
  const rawTitle = (session?.display_title || session?.title || "").trim();
  const title = sanitizeSegment(rawTitle) || "session-export";
  return `${title}-${formatTimestamp(now)}.md`;
}
