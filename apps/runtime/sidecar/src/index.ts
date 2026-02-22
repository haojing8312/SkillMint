import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { serve } from '@hono/node-server';
import { BrowserController } from './browser.js';
import { MCPManager } from './mcp.js';
import type { ApiResponse } from './types.js';

const app = new Hono();
const browser = new BrowserController();
const mcp = new MCPManager();

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

// MCP endpoints
app.post('/api/mcp/add-server', async (c) => {
  try {
    const { name, command, args, env } = await c.req.json();
    await mcp.addServer(name, { command, args, env });
    return c.json({ output: `MCP 服务器 ${name} 已添加` } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/mcp/list-servers', async (c) => {
  try {
    const servers = mcp.listServers();
    return c.json({ output: JSON.stringify(servers) } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/mcp/call-tool', async (c) => {
  try {
    const { server_name, tool_name, arguments: args } = await c.req.json();
    const result = await mcp.callTool(server_name, tool_name, args);
    return c.json({ output: JSON.stringify(result) } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

app.post('/api/mcp/list-tools', async (c) => {
  try {
    const { server_name } = await c.req.json();
    const tools = await mcp.listTools(server_name);
    return c.json({ output: JSON.stringify(tools) } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

// Web search endpoint (DuckDuckGo HTML)
app.post('/api/web/search', async (c) => {
  try {
    const { query, count = 5 } = await c.req.json();
    const url = `https://html.duckduckgo.com/html/?q=${encodeURIComponent(query)}`;
    const resp = await fetch(url, {
      headers: { 'User-Agent': 'SkillMint/1.0' },
    });
    const html = await resp.text();

    // 解析 DuckDuckGo HTML 结果
    const results: { title: string; url: string; snippet: string }[] = [];
    const resultRegex = /<a[^>]*class="result__a"[^>]*href="([^"]*)"[^>]*>(.*?)<\/a>[\s\S]*?<a[^>]*class="result__snippet"[^>]*>(.*?)<\/a>/gi;
    let match;
    while ((match = resultRegex.exec(html)) !== null && results.length < count) {
      results.push({
        url: match[1].replace(/\/\/duckduckgo\.com\/l\/\?uddg=/, '').split('&')[0],
        title: match[2].replace(/<[^>]+>/g, '').trim(),
        snippet: match[3].replace(/<[^>]+>/g, '').trim(),
      });
    }

    // 如果正则没匹配到，尝试简单解析
    if (results.length === 0) {
      const linkRegex = /<a[^>]*class="result__a"[^>]*href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/gi;
      while ((match = linkRegex.exec(html)) !== null && results.length < count) {
        const rawUrl = match[1];
        const decodedUrl = rawUrl.includes('uddg=')
          ? decodeURIComponent(rawUrl.split('uddg=')[1]?.split('&')[0] || rawUrl)
          : rawUrl;
        results.push({
          url: decodedUrl,
          title: match[2].replace(/<[^>]+>/g, '').trim(),
          snippet: '',
        });
      }
    }

    const output = results
      .map((r, i) => `${i + 1}. ${r.title}\n   ${r.url}\n   ${r.snippet}`)
      .join('\n\n');

    return c.json({ output: output || '未找到搜索结果' } as ApiResponse);
  } catch (e: any) {
    return c.json({ error: e.message } as ApiResponse, 500);
  }
});

const PORT = Number(process.env.PORT || 8765);
console.log(`[sidecar] Starting on http://localhost:${PORT}`);

// Graceful shutdown
process.on('SIGINT', async () => {
  await browser.close();
  await mcp.closeAll();
  process.exit(0);
});

serve({ fetch: app.fetch, port: PORT });

export default app;
