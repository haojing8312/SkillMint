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
