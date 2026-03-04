import test from "node:test";
import assert from "node:assert/strict";
import app from "../src/index.js";

test("feishu ws reconcile endpoint returns status summary shape", async () => {
  const res = await app.fetch(
    new Request("http://localhost/api/feishu/ws/reconcile", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ employees: [] }),
    }),
  );
  assert.equal(res.status, 200);
  const json = await res.json();
  const output = JSON.parse(String(json.output || "null"));
  assert.ok(Array.isArray(output.items));
  assert.equal(output.items.length, 0);
});

test("feishu ws status endpoint exposes per-employee items", async () => {
  const res = await app.fetch(
    new Request("http://localhost/api/feishu/ws/status", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({}),
    }),
  );
  assert.equal(res.status, 200);
  const json = await res.json();
  const output = JSON.parse(String(json.output || "null"));
  assert.ok("items" in output);
  assert.ok(Array.isArray(output.items));
});
