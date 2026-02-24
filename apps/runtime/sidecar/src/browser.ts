import { chromium, Browser, BrowserContext, Page } from 'playwright';

/** 浏览器启动选项 */
export interface LaunchOptions {
  headless?: boolean;
  viewport?: { width: number; height: number };
}

/** DOM 节点的简化表示 */
interface DOMNode {
  tag: string;
  id?: string;
  class?: string;
  text?: string;
  children?: DOMNode[];
}

/** 等待条件选项 */
export interface WaitForOptions {
  /** CSS 选择器，等待元素出现 */
  selector?: string;
  /** JavaScript 条件表达式，等待返回 truthy */
  condition?: string;
  /** 超时时间（毫秒），默认 30000 */
  timeout?: number;
}

/** 浏览器状态信息 */
interface BrowserState {
  running: boolean;
  url: string | null;
  title: string | null;
}

/**
 * 浏览器自动化控制器
 *
 * 封装 Playwright 提供完整的浏览器操作能力，
 * 内置 stealth 反检测技术隐藏自动化特征。
 */
export class BrowserController {
  private browser: Browser | null = null;
  private context: BrowserContext | null = null;
  private page: Page | null = null;

  // ─── stealth 反检测脚本 ───────────────────────────────────────────
  // 参考 puppeteer-extra-plugin-stealth 的核心技术，
  // 通过 addInitScript 在页面加载前注入，隐藏自动化特征。
  private static readonly STEALTH_SCRIPT = `
    // 1. 隐藏 navigator.webdriver 标志
    Object.defineProperty(navigator, 'webdriver', {
      get: () => undefined,
    });

    // 2. 伪造 navigator.languages 看起来像真实浏览器
    Object.defineProperty(navigator, 'languages', {
      get: () => ['zh-CN', 'zh', 'en-US', 'en'],
    });

    // 3. 伪造 navigator.plugins 使其非空（无头浏览器通常为空）
    Object.defineProperty(navigator, 'plugins', {
      get: () => {
        const plugins = [
          { name: 'Chrome PDF Plugin', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
          { name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai', description: '' },
          { name: 'Native Client', filename: 'internal-nacl-plugin', description: '' },
        ];
        // 模拟 PluginArray 接口
        const pluginArray = Object.create(PluginArray.prototype);
        plugins.forEach((p, i) => {
          const plugin = Object.create(Plugin.prototype);
          Object.defineProperties(plugin, {
            name: { value: p.name, enumerable: true },
            filename: { value: p.filename, enumerable: true },
            description: { value: p.description, enumerable: true },
            length: { value: 0, enumerable: true },
          });
          pluginArray[i] = plugin;
        });
        Object.defineProperty(pluginArray, 'length', { value: plugins.length });
        return pluginArray;
      },
    });

    // 4. 隐藏 Playwright/Chromium 自动化相关的 window 属性
    delete (window as any).__playwright;
    delete (window as any).__pw_manual;

    // 5. 覆盖 navigator.permissions.query 防止泄露自动化状态
    const originalQuery = window.navigator.permissions.query.bind(window.navigator.permissions);
    window.navigator.permissions.query = (parameters: any) => {
      if (parameters.name === 'notifications') {
        return Promise.resolve({ state: Notification.permission } as PermissionStatus);
      }
      return originalQuery(parameters);
    };

    // 6. 伪造 chrome runtime 对象（Chrome 浏览器特有）
    if (!(window as any).chrome) {
      (window as any).chrome = {};
    }
    if (!(window as any).chrome.runtime) {
      (window as any).chrome.runtime = {
        connect: () => {},
        sendMessage: () => {},
      };
    }

    // 7. 覆盖 WebGL 渲染器信息，避免被指纹识别
    const getParameter = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(parameter: number) {
      // UNMASKED_VENDOR_WEBGL
      if (parameter === 37445) return 'Google Inc. (Intel)';
      // UNMASKED_RENDERER_WEBGL
      if (parameter === 37446) return 'ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)';
      return getParameter.call(this, parameter);
    };
  `;

  /**
   * 确保浏览器已启动，若未启动则使用默认配置启动。
   * 会自动注入 stealth 反检测脚本。
   */
  private async ensureBrowser() {
    if (!this.browser) {
      await this.launch();
    }
  }

  /**
   * 应用 stealth 脚本到浏览器上下文
   */
  private async applyStealthToContext(ctx: BrowserContext): Promise<void> {
    await ctx.addInitScript(BrowserController.STEALTH_SCRIPT);
  }

  // ─── 新增方法 ─────────────────────────────────────────────────────

  /**
   * 启动浏览器实例
   *
   * @param options - 启动选项（headless、viewport）
   * @returns 启动成功的提示消息
   */
  async launch(options?: LaunchOptions): Promise<string> {
    // 如果浏览器已在运行，先关闭旧实例
    if (this.browser) {
      await this.close();
    }

    const headless = options?.headless ?? false;
    const viewport = options?.viewport ?? { width: 1280, height: 720 };

    this.browser = await chromium.launch({
      headless,
      args: [
        // stealth 相关启动参数
        '--disable-blink-features=AutomationControlled',
        '--disable-infobars',
        '--no-first-run',
        '--no-default-browser-check',
      ],
    });

    this.context = await this.browser.newContext({
      viewport,
      userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
      locale: 'zh-CN',
    });

    // 注入 stealth 反检测脚本
    await this.applyStealthToContext(this.context);

    this.page = await this.context.newPage();

    return `浏览器已启动 (headless=${headless}, viewport=${viewport.width}x${viewport.height})`;
  }

  /**
   * 导航到指定 URL
   */
  async navigate(url: string): Promise<string> {
    await this.ensureBrowser();
    await this.page!.goto(url, { waitUntil: 'domcontentloaded' });
    return `已导航到 ${url}`;
  }

  /**
   * 点击指定元素
   */
  async click(selector: string): Promise<string> {
    await this.ensureBrowser();
    await this.page!.click(selector);
    return `已点击 ${selector}`;
  }

  /**
   * 在指定元素中输入文本
   *
   * @param selector - CSS 选择器
   * @param text - 要输入的文本
   * @param delay - 每个字符之间的延迟（毫秒），模拟人类打字速度
   */
  async type(selector: string, text: string, delay?: number): Promise<string> {
    await this.ensureBrowser();
    // 先点击聚焦目标元素
    await this.page!.click(selector);
    await this.page!.type(selector, text, { delay: delay ?? 0 });
    return `已在 ${selector} 中输入 ${text.length} 个字符`;
  }

  /**
   * 滚动页面
   *
   * @param direction - 滚动方向：up / down / to_top / to_bottom
   * @param amount - 滚动像素量（仅 up/down 生效），默认 500
   */
  async scroll(direction: string, amount?: number): Promise<string> {
    await this.ensureBrowser();
    const px = amount ?? 500;

    switch (direction) {
      case 'up':
        await this.page!.evaluate((p: number) => window.scrollBy(0, -p), px);
        return `已向上滚动 ${px}px`;
      case 'down':
        await this.page!.evaluate((p: number) => window.scrollBy(0, p), px);
        return `已向下滚动 ${px}px`;
      case 'to_top':
        await this.page!.evaluate(() => window.scrollTo(0, 0));
        return '已滚动到页面顶部';
      case 'to_bottom':
        await this.page!.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
        return '已滚动到页面底部';
      default:
        throw new Error(`不支持的滚动方向: ${direction}，可选: up/down/to_top/to_bottom`);
    }
  }

  /**
   * 悬停在指定元素上
   */
  async hover(selector: string): Promise<string> {
    await this.ensureBrowser();
    await this.page!.hover(selector);
    return `已悬停在 ${selector}`;
  }

  /**
   * 按下键盘按键
   *
   * @param key - 按键名称（如 Enter、Tab、ArrowDown 等）
   * @param modifiers - 修饰键数组（如 ['Control', 'Shift']）
   */
  async pressKey(key: string, modifiers?: string[]): Promise<string> {
    await this.ensureBrowser();

    // 构建组合键字符串，如 "Control+Shift+A"
    const combo = modifiers && modifiers.length > 0
      ? [...modifiers, key].join('+')
      : key;

    await this.page!.keyboard.press(combo);
    return `已按下 ${combo}`;
  }

  /**
   * 获取页面的简化 DOM 结构
   *
   * @param selector - 起始节点的 CSS 选择器，默认为 body
   * @param maxDepth - 最大遍历深度，默认 3
   * @returns JSON 格式的简化 DOM 树
   */
  async getDOM(selector?: string, maxDepth?: number): Promise<string> {
    await this.ensureBrowser();
    const sel = selector ?? 'body';
    const depth = maxDepth ?? 3;

    const dom = await this.page!.evaluate(
      ({ sel, depth }: { sel: string; depth: number }) => {
        // 在页面上下文中递归提取 DOM 结构
        function extractNode(el: Element, currentDepth: number): Record<string, unknown> {
          const node: Record<string, unknown> = { tag: el.tagName.toLowerCase() };

          if (el.id) node.id = el.id;

          const classes = el.className;
          if (typeof classes === 'string' && classes.trim()) {
            node.class = classes.trim();
          }

          // 提取直接文本内容（不含子元素的文本）
          const directText = Array.from(el.childNodes)
            .filter((n: ChildNode) => n.nodeType === Node.TEXT_NODE)
            .map((n: ChildNode) => n.textContent?.trim())
            .filter(Boolean)
            .join(' ');
          if (directText) {
            node.text = directText.substring(0, 200);
          }

          // 递归子元素
          if (currentDepth < depth) {
            const children: Record<string, unknown>[] = [];
            for (const child of Array.from(el.children)) {
              // 跳过 script、style、svg 等不需要的元素
              const tag = (child as Element).tagName.toLowerCase();
              if (['script', 'style', 'noscript', 'svg', 'path'].includes(tag)) continue;
              children.push(extractNode(child as Element, currentDepth + 1));
            }
            if (children.length > 0) {
              node.children = children;
            }
          }

          return node;
        }

        const root = document.querySelector(sel);
        if (!root) return null;
        return extractNode(root, 0);
      },
      { sel, depth },
    );

    if (!dom) {
      throw new Error(`未找到匹配 "${sel}" 的元素`);
    }

    return JSON.stringify(dom, null, 2);
  }

  /**
   * 等待指定条件满足
   *
   * @param options - 等待选项（selector 或 condition，以及 timeout）
   */
  async waitFor(options: WaitForOptions): Promise<string> {
    await this.ensureBrowser();
    const timeout = options.timeout ?? 30000;

    if (options.selector) {
      await this.page!.waitForSelector(options.selector, { timeout });
      return `元素 ${options.selector} 已出现`;
    }

    if (options.condition) {
      await this.page!.waitForFunction(options.condition, undefined, { timeout });
      return `条件已满足: ${options.condition}`;
    }

    throw new Error('waitFor 需要提供 selector 或 condition 参数');
  }

  /**
   * 导航后退
   */
  async goBack(): Promise<string> {
    await this.ensureBrowser();
    await this.page!.goBack({ waitUntil: 'domcontentloaded' });
    const url = this.page!.url();
    return `已后退到 ${url}`;
  }

  /**
   * 导航前进
   */
  async goForward(): Promise<string> {
    await this.ensureBrowser();
    await this.page!.goForward({ waitUntil: 'domcontentloaded' });
    const url = this.page!.url();
    return `已前进到 ${url}`;
  }

  /**
   * 刷新当前页面
   */
  async reload(): Promise<string> {
    await this.ensureBrowser();
    await this.page!.reload({ waitUntil: 'domcontentloaded' });
    return `已刷新页面: ${this.page!.url()}`;
  }

  /**
   * 获取浏览器当前状态
   */
  async getState(): Promise<string> {
    const state: BrowserState = {
      running: this.browser !== null,
      url: this.page ? this.page.url() : null,
      title: this.page ? await this.page.title() : null,
    };
    return JSON.stringify(state);
  }

  /**
   * 截取页面截图
   */
  async screenshot(path: string): Promise<string> {
    await this.ensureBrowser();
    await this.page!.screenshot({ path, fullPage: true });
    return `截图已保存到 ${path}`;
  }

  /**
   * 在页面中执行 JavaScript
   */
  async evaluate(script: string): Promise<string> {
    await this.ensureBrowser();
    const result = await this.page!.evaluate(script);
    return JSON.stringify(result);
  }

  /**
   * 获取页面 HTML 内容
   */
  async getContent(): Promise<string> {
    await this.ensureBrowser();
    return await this.page!.content();
  }

  /**
   * 关闭浏览器实例并释放资源
   */
  async close() {
    if (this.browser) {
      await this.browser.close();
      this.browser = null;
      this.context = null;
      this.page = null;
    }
  }
}
