import { chromium, Browser, Page } from 'playwright';

export class BrowserController {
  private browser: Browser | null = null;
  private page: Page | null = null;

  private async ensureBrowser() {
    if (!this.browser) {
      this.browser = await chromium.launch({ headless: false });
      const context = await this.browser.newContext();
      this.page = await context.newPage();
    }
  }

  async navigate(url: string): Promise<string> {
    await this.ensureBrowser();
    await this.page!.goto(url, { waitUntil: 'domcontentloaded' });
    return `已导航到 ${url}`;
  }

  async click(selector: string): Promise<string> {
    await this.ensureBrowser();
    await this.page!.click(selector);
    return `已点击 ${selector}`;
  }

  async screenshot(path: string): Promise<string> {
    await this.ensureBrowser();
    await this.page!.screenshot({ path, fullPage: true });
    return `截图已保存到 ${path}`;
  }

  async evaluate(script: string): Promise<string> {
    await this.ensureBrowser();
    const result = await this.page!.evaluate(script);
    return JSON.stringify(result);
  }

  async getContent(): Promise<string> {
    await this.ensureBrowser();
    return await this.page!.content();
  }

  async close() {
    if (this.browser) {
      await this.browser.close();
      this.browser = null;
      this.page = null;
    }
  }
}
