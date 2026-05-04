import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const projectRoot = process.cwd();
const packageJsonPath = path.join(projectRoot, "package.json");

test("production audit overrides pin transitive advisory fixes", () => {
  const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));
  const overrides = pkg.pnpm?.overrides ?? {};

  assert.equal(overrides.prismjs, "1.30.0");
  assert.equal(overrides["@modelcontextprotocol/sdk>@hono/node-server"], "1.19.13");
  assert.equal(overrides["@larksuiteoapi/node-sdk>axios"], "1.16.0");
  assert.equal(overrides["axios>follow-redirects"], "1.16.0");
});
