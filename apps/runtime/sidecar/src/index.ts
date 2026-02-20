import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { serve } from '@hono/node-server';

const app = new Hono();

app.use('/*', cors());

app.get('/health', (c) => {
  return c.json({ status: 'ok', uptime: process.uptime() });
});

const PORT = Number(process.env.PORT || 8765);
console.log(`[sidecar] Starting on http://localhost:${PORT}`);

serve({ fetch: app.fetch, port: PORT });

export default app;
