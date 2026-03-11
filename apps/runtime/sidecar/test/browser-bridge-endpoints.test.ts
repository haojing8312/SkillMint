import test from 'node:test';
import assert from 'node:assert/strict';
import { createSidecarApp } from '../src/index.js';

test('browser bridge endpoint accepts credentials report envelopes', async () => {
  const app = createSidecarApp();

  const res = await app.fetch(
    new Request('http://localhost/api/browser-bridge/native-message', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        version: 1,
        sessionId: 'sess-bridge-1',
        kind: 'request',
        payload: {
          type: 'credentials.report',
          appId: 'cli_123',
          appSecret: 'sec_456',
        },
      }),
    }),
  );

  assert.equal(res.status, 200);
  const json = await res.json();
  assert.deepEqual(json, {
    version: 1,
    sessionId: 'sess-bridge-1',
    kind: 'response',
    payload: {
      type: 'action.pause',
      reason: 'browser bridge credentials received',
    },
  });
});

test('browser bridge endpoint rejects invalid envelopes', async () => {
  const app = createSidecarApp();

  const res = await app.fetch(
    new Request('http://localhost/api/browser-bridge/native-message', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        sessionId: 'sess-bridge-2',
      }),
    }),
  );

  assert.equal(res.status, 400);
  const json = await res.json();
  assert.equal(typeof json.error, 'string');
});
