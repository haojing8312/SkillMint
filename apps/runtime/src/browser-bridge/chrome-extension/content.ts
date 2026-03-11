import { detectFeishuPage } from "./feishu-detector";

export function detectCurrentFeishuPage(doc: Document = document) {
  return detectFeishuPage(doc);
}

export function extractFeishuCredentials(doc: Document = document): {
  appId: string;
  appSecret: string;
} | null {
  const appId =
    doc.querySelector("[data-field='app-id']")?.textContent?.trim() ??
    findValueNearLabel(doc, "App ID");
  const appSecret =
    doc.querySelector("[data-field='app-secret']")?.textContent?.trim() ??
    findValueNearLabel(doc, "App Secret");

  if (!appId || !appSecret) {
    return null;
  }

  return {
    appId,
    appSecret,
  };
}

type ChromeLike = {
  runtime?: {
    sendMessage?: (message: unknown) => Promise<unknown> | unknown;
  };
};

export function getFeishuBrowserSetupSessionId(href: string = window.location.href): string | null {
  const url = new URL(href);
  return url.searchParams.get("workclaw_session_id");
}

export async function maybeReportFeishuCredentialsToExtension(
  locationLike: Pick<Location, "href"> = window.location,
  doc: Document = document,
  chromeLike: ChromeLike = globalThis as ChromeLike,
): Promise<boolean> {
  const sessionId = getFeishuBrowserSetupSessionId(locationLike.href);
  if (!sessionId) {
    return false;
  }

  if (detectCurrentFeishuPage(doc).kind !== "credentials") {
    return false;
  }

  const credentials = extractFeishuCredentials(doc);
  if (!credentials) {
    return false;
  }

  await chromeLike.runtime?.sendMessage?.({
    type: "workclaw.report-feishu-credentials",
    sessionId,
    appId: credentials.appId,
    appSecret: credentials.appSecret,
  });

  return true;
}

export function installFeishuCredentialReporter(
  locationLike: Pick<Location, "href"> = window.location,
  doc: Document = document,
  chromeLike: ChromeLike = globalThis as ChromeLike,
): () => void {
  const sessionId = getFeishuBrowserSetupSessionId(locationLike.href);
  if (!sessionId) {
    return () => {};
  }

  let disposed = false;
  let reported = false;
  const tryReport = async () => {
    if (disposed || reported) {
      return;
    }

    reported = await maybeReportFeishuCredentialsToExtension(locationLike, doc, chromeLike);
    if (reported) {
      observer.disconnect();
    }
  };

  const observer = new MutationObserver(() => {
    void tryReport();
  });
  const root = doc.documentElement ?? doc.body;
  if (root) {
    observer.observe(root, {
      childList: true,
      subtree: true,
      characterData: true,
      attributes: true,
    });
  }

  void tryReport();

  return () => {
    disposed = true;
    observer.disconnect();
  };
}

export async function initializeFeishuContentScript(
  locationLike: Pick<Location, "href"> = window.location,
  doc: Document = document,
  chromeLike: ChromeLike = globalThis as ChromeLike,
): Promise<boolean> {
  installFeishuCredentialReporter(locationLike, doc, chromeLike);
  return false;
}

function findValueNearLabel(doc: Document, label: string): string {
  const elements = Array.from(doc.querySelectorAll("div, span, p, td, dt, dd, label"));
  const labelElement = elements
    .filter((element) => normalizeText(element.textContent) === label)
    .sort((left, right) => {
      const depthDifference = getElementDepth(right) - getElementDepth(left);
      if (depthDifference !== 0) {
        return depthDifference;
      }
      return left.children.length - right.children.length;
    })[0];
  if (!labelElement) {
    return "";
  }

  let sibling = labelElement.nextElementSibling;
  while (sibling) {
    const text = firstMeaningfulText(sibling, label);
    if (text && text !== label) {
      return text;
    }
    sibling = sibling.nextElementSibling;
  }

  const parentValue = findValueInParentBlock(labelElement, label);
  if (parentValue) {
    return parentValue;
  }

  return "";
}

function findValueInParentBlock(labelElement: Element, label: string): string {
  const parent = labelElement.parentElement;
  if (!parent) {
    return "";
  }

  const children = Array.from(parent.children);
  const labelIndex = children.indexOf(labelElement);
  for (let index = labelIndex + 1; index < children.length; index += 1) {
    const text = firstMeaningfulText(children[index] as Element, label);
    if (text && text !== label) {
      return text;
    }
  }

  return "";
}

function normalizeText(value: string | null | undefined): string {
  return (value ?? "").replace(/\s+/g, " ").trim();
}

function getElementDepth(element: Element): number {
  let depth = 0;
  let current = element.parentElement;
  while (current) {
    depth += 1;
    current = current.parentElement;
  }
  return depth;
}

function firstMeaningfulText(element: Element, label: string): string {
  const fieldValue = readFieldValue(element);
  if (fieldValue && fieldValue !== label) {
    return fieldValue;
  }

  const ownText = normalizeText(element.textContent);
  if (ownText && ownText !== label && ownText !== "凭证与基础信息") {
    return ownText;
  }

  const descendants = Array.from(element.querySelectorAll("div, span, p, td, dt, dd, label"));
  for (const descendant of descendants) {
    const text = normalizeText(descendant.textContent);
    if (text && text !== label && text !== "凭证与基础信息") {
      return text;
    }
  }

  return "";
}

function readFieldValue(element: Element): string {
  if ("value" in element && typeof element.value === "string") {
    return normalizeText(element.value);
  }

  const field = element.querySelector("input, textarea");
  if (field && "value" in field && typeof field.value === "string") {
    return normalizeText(field.value);
  }

  return "";
}

void initializeFeishuContentScript().catch(() => {
  // Ignore runtime bridge failures in the passive content script path.
});
