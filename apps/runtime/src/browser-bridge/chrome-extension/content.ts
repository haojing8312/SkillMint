import { detectFeishuPage } from "./feishu-detector";

export function detectCurrentFeishuPage(doc: Document = document) {
  return detectFeishuPage(doc);
}
