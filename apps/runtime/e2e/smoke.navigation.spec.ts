import { expect, type Page, test } from "@playwright/test";

type TauriMockSkill = {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  recommended_model: string;
  tags: string[];
  created_at: string;
};

type TauriMockModel = {
  id: string;
  name: string;
  api_format: string;
  base_url: string;
  model_name: string;
  is_default: boolean;
};

async function installTauriMocks(page: Page): Promise<void> {
  const skills: TauriMockSkill[] = [
    {
      id: "builtin-general",
      name: "General",
      description: "Default skill",
      version: "1.0.0",
      author: "e2e",
      recommended_model: "model-a",
      tags: [],
      created_at: new Date().toISOString(),
    },
  ];
  const models: TauriMockModel[] = [
    {
      id: "model-a",
      name: "OpenAI",
      api_format: "openai",
      base_url: "https://api.openai.com/v1",
      model_name: "gpt-4o-mini",
      is_default: true,
    },
  ];

  await page.addInitScript(
    ({ mockedSkills, mockedModels }) => {
      const runtimePreferences = {
        default_work_dir: "",
        default_language: "zh-CN",
        immersive_translation_enabled: true,
        immersive_translation_display: "translated_only",
      };

      const providerConfig = {
        id: mockedModels[0]?.id || "model-a",
        provider_key: "openai",
        display_name: mockedModels[0]?.name || "OpenAI",
        protocol_type: "openai",
        base_url: mockedModels[0]?.base_url || "https://api.openai.com/v1",
        auth_type: "api_key",
        api_key_encrypted: "***",
        org_id: "",
        extra_json: "{}",
        enabled: true,
      };

      const invoke = async (cmd: string, args?: Record<string, unknown>) => {
        switch (cmd) {
          case "list_skills":
            return mockedSkills;
          case "list_model_configs":
            return mockedModels;
          case "list_agent_employees":
            return [];
          case "get_sessions":
            return [];
          case "list_search_configs":
            return [];
          case "get_runtime_preferences":
            return runtimePreferences;
          case "list_mcp_servers":
            return [];
          case "get_model_api_key":
            return "sk-e2e-mock";
          case "save_provider_config":
            return null;
          case "list_provider_configs":
            return [providerConfig];
          case "set_runtime_preferences":
            return {
              ...runtimePreferences,
              ...(args?.input as Record<string, unknown> | undefined),
            };
          default:
            return null;
        }
      };

      const w = window as typeof window & {
        __TAURI_INTERNALS__?: { invoke: typeof invoke };
      };
      w.__TAURI_INTERNALS__ = { invoke };
    },
    { mockedSkills: skills, mockedModels: models },
  );
}

test("main navigation smoke flow works end-to-end", async ({ page }) => {
  await installTauriMocks(page);
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "你的电脑任务，交给打工虾们协作完成" }),
  ).toBeVisible();

  await page.getByRole("button", { name: "设置" }).first().click();
  await expect(page.getByRole("button", { name: "模型连接" })).toBeVisible();

  await page.getByRole("button", { name: "专家技能" }).first().click();
  await expect(page.getByRole("heading", { name: "专家技能" })).toBeVisible();

  await page.getByRole("button", { name: "开始任务" }).first().click();
  await expect(
    page.getByRole("heading", { name: "你的电脑任务，交给打工虾们协作完成" }),
  ).toBeVisible();
});
