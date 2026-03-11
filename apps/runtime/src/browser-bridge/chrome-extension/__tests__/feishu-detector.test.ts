import { describe, expect, it } from "vitest";
import { detectFeishuPage } from "../feishu-detector";

describe("detectFeishuPage", () => {
  it("detects logged-out state", () => {
    document.body.innerHTML = `<button>登录</button>`;
    expect(detectFeishuPage(document).kind).toBe("login");
  });

  it("detects credential page", () => {
    document.body.innerHTML = `<div>凭证与基础信息</div><div>App ID</div><div>App Secret</div>`;
    expect(detectFeishuPage(document).kind).toBe("credentials");
  });
});
