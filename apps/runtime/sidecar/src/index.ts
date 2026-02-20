import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { serve } from '@hono/node-server';
import { BrowserController } from './browser';
import type { ApiResponse } from './types';

const app = new Hono();
const browser = new BrowserController();

app.use('/*', cors());

app.get('/health', (c) => {
  return c.json({ status: 'ok', uptime: process.uptime() });
});

// Browser endpoints
app.post('/api/browser/navigate', async (c) => {
  try {
    const { url } = await c.req.json();
    const result = await browser.navigate(url);
    return c.json({ output: result } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/browser/click', async (c) => {
  try {
    const { selector } = await c.req.json();
    const result = await browser.click(selector);
    return c.json({ output: result } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/browser/screenshot', async (c) => {
  try {
    const { path } = await c.req.json();
    const result = await browser.screenshot(path);
    return c.json({ output: result } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/browser/evaluate', async (c) => {
  try {
    const { script } = await c.req.json();
    const result = await browser.evaluate(script);
    return c.json({ output: result } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/browser/content', async (c) => {
  try {
    const result = await browser.getContent();
    return c.json({ output: result } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/browser/close', async (c) => {
  try {
    await browser.close();
    return c.json({ output: '浏览器已关闭' } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

const PORT = Number(process.env.PORT || 8765);
console.log(`[sidecar] Starting on http://localhost:${PORT}`);

// Graceful shutdown
process.on('SIGINT', async () => {
  await browser.close();
  process.exit(0);
});

serve({ fetch: app.fetch, port: PORT });

export default app;
