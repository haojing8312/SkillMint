import { describe, expect, it } from "vitest";
import { detectCurrentFeishuPage, extractFeishuCredentials } from "../content";

describe("chrome extension content helpers", () => {
  it("detects the current Feishu page", () => {
    document.body.innerHTML = `<div>凭证与基础信息</div><div>App ID</div>`;
    expect(detectCurrentFeishuPage(document).kind).toBe("credentials");
  });

  it("extracts App ID and App Secret from the credential page", () => {
    document.body.innerHTML = `
      <div>凭证与基础信息</div>
      <div data-field="app-id">cli_123</div>
      <div data-field="app-secret">sec_456</div>
    `;

    expect(extractFeishuCredentials(document)).toEqual({
      appId: "cli_123",
      appSecret: "sec_456",
    });
  });

  it("extracts credentials from label-and-value blocks", () => {
    document.body.innerHTML = `
      <section>
        <div class="field">
          <div class="label">App ID</div>
          <div class="value">cli_label_123</div>
        </div>
        <div class="field">
          <div class="label">App Secret</div>
          <div class="value">sec_label_456</div>
        </div>
      </section>
    `;

    expect(extractFeishuCredentials(document)).toEqual({
      appId: "cli_label_123",
      appSecret: "sec_label_456",
    });
  });

  it("extracts credentials from adjacent text when the values are not marked", () => {
    document.body.innerHTML = `
      <section>
        <div>凭证与基础信息</div>
        <div>App ID</div>
        <div>cli_adjacent_123</div>
        <div>App Secret</div>
        <div>sec_adjacent_456</div>
      </section>
    `;

    expect(extractFeishuCredentials(document)).toEqual({
      appId: "cli_adjacent_123",
      appSecret: "sec_adjacent_456",
    });
  });
});
