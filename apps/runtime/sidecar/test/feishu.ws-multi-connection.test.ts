import test from "node:test";
import assert from "node:assert/strict";
import {
  FeishuLongConnectionManager,
  type FeishuWsEventRecord,
} from "../src/feishu_ws.js";

class FakeEventDispatcher {
  handlers: Record<string, (payload: unknown) => Promise<unknown>> = {};

  register(handlers: Record<string, (payload: unknown) => Promise<unknown>>) {
    this.handlers = handlers;
    return this;
  }
}

class FakeWSClient {
  static instances: FakeWSClient[] = [];

  config: Record<string, unknown>;
  dispatcher: FakeEventDispatcher | null = null;
  started = false;
  stopped = false;
  closed = false;

  constructor(config: Record<string, unknown>) {
    this.config = config;
    FakeWSClient.instances.push(this);
  }

  start({ eventDispatcher }: { eventDispatcher: FakeEventDispatcher }) {
    this.dispatcher = eventDispatcher;
    this.started = true;
  }

  stop() {
    this.stopped = true;
  }

  close() {
    this.closed = true;
  }
}

const fakeSdk = {
  WSClient: FakeWSClient as unknown as new (config: Record<string, unknown>) => unknown,
  EventDispatcher: FakeEventDispatcher as unknown as new (
    options: Record<string, unknown>,
  ) => FakeEventDispatcher,
  LoggerLevel: { info: 1 },
};

test("reconcile keeps multiple employee long-connections running", () => {
  const manager = new FeishuLongConnectionManager(fakeSdk);
  const summary = manager.reconcile([
    { employee_id: "project_manager", app_id: "cli_pm", app_secret: "sec_pm" },
    { employee_id: "tech_lead", app_id: "cli_tl", app_secret: "sec_tl" },
  ]);

  assert.equal(summary.items.length, 2);
  assert.equal(summary.items.every((item) => item.running), true);
  assert.equal(FakeWSClient.instances.length, 2);
});

test("drain keeps employee_id on ws events", async () => {
  const manager = new FeishuLongConnectionManager(fakeSdk);
  manager.reconcile([{ employee_id: "project_manager", app_id: "cli_pm", app_secret: "sec_pm" }]);

  const ws = FakeWSClient.instances.at(-1);
  assert.ok(ws?.dispatcher);
  await ws!.dispatcher!.handlers["im.message.receive_v1"]({
    message: {
      chat_id: "oc_chat_1",
      message_id: "om_1",
      content: "{\"text\":\"hello\"}",
    },
    sender: {
      sender_id: { open_id: "ou_xxx" },
    },
  });

  const events = manager.drainAll(10);
  assert.equal(events.length, 1);
  const first = events[0] as FeishuWsEventRecord;
  assert.equal(first.employee_id, "project_manager");
  assert.equal(first.chat_id, "oc_chat_1");
  assert.equal(first.text, "hello");
});

test("coalesces duplicate events by chat/message and keeps mention target", async () => {
  const manager = new FeishuLongConnectionManager(fakeSdk);
  manager.reconcile([
    { employee_id: "project_manager", app_id: "cli_pm", app_secret: "sec_pm" },
    { employee_id: "dev_team", app_id: "cli_dev", app_secret: "sec_dev" },
  ]);

  const instances = FakeWSClient.instances.slice(-2);
  const wsProjectManager = instances[0];
  const wsDevTeam = instances[1];
  assert.ok(wsProjectManager?.dispatcher);
  assert.ok(wsDevTeam?.dispatcher);

  const payloadWithoutMention = {
    message: {
      chat_id: "oc_chat_same",
      message_id: "om_same",
      content: "{\"text\":\"@_user_1 你细化一下技术方案\"}",
    },
    sender: {
      sender_id: { open_id: "ou_sender_1" },
    },
  };
  const payloadWithMention = {
    ...payloadWithoutMention,
    mentions: [
      {
        key: "@_user_1",
        id: { open_id: "ou_dev_team" },
        name: "开发团队",
      },
    ],
  };

  // First event may come from non-targeted connection without usable mention metadata.
  await wsProjectManager!.dispatcher!.handlers["im.message.receive_v1"](payloadWithoutMention);
  // Second event (same chat/message id) carries the real @ target and should enrich the queue record.
  await wsDevTeam!.dispatcher!.handlers["im.message.receive_v1"](payloadWithMention);

  const events = manager.drainAll(10);
  assert.equal(events.length, 1);
  assert.equal(events[0].id, "oc_chat_same:om_same");
  assert.equal(events[0].mention_open_id, "ou_dev_team");
  assert.equal(events[0].text, "你细化一下技术方案");
  assert.deepEqual(events[0].source_employee_ids.sort(), ["dev_team", "project_manager"]);
});
