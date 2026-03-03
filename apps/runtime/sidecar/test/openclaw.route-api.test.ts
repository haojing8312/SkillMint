import test from "node:test";
import assert from "node:assert/strict";
import app from "../src/index.js";

test("route resolve endpoint returns matched route", async () => {
  const req = new Request("http://localhost/api/openclaw/resolve-route", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      channel: "feishu",
      account_id: "acct-a",
      peer: { kind: "group", id: "chat-1" },
      default_agent_id: "main",
      bindings: [{ agentId: "main", match: { channel: "feishu", accountId: "*" } }],
    }),
  });
  const res = await app.fetch(req);
  const json = await res.json();
  assert.equal(Boolean(json.output), true);
});
