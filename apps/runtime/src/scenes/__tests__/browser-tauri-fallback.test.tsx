import { act, renderHook } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";

import { useCatalogDataCoordinator } from "../useCatalogDataCoordinator";
import { useRuntimePreferencesCoordinator } from "../useRuntimePreferencesCoordinator";
import {
  DEFAULT_RUNTIME_PREFERENCES,
  getRuntimePreferences,
} from "../../components/settings/desktop/desktopSettingsService";
import {
  listModelConfigs,
  listProviderConfigs,
} from "../../components/settings/models/modelSettingsService";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const missingTauriInvokeError = new TypeError(
  "Cannot read properties of undefined (reading 'invoke')",
);

describe("browser-only Tauri fallback", () => {
  let warnSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    (window as typeof window & { __WORKCLAW_FORCE_BROWSER_ONLY__?: boolean }).__WORKCLAW_FORCE_BROWSER_ONLY__ = true;
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockImplementation(() => {
      throw missingTauriInvokeError;
    });
    warnSpy = vi.spyOn(console, "warn").mockImplementation(() => undefined);
  });

  afterEach(() => {
    delete (window as typeof window & { __WORKCLAW_FORCE_BROWSER_ONLY__?: boolean }).__WORKCLAW_FORCE_BROWSER_ONLY__;
    warnSpy.mockRestore();
  });

  test("catalog startup loaders resolve empty state instead of surfacing unhandled invoke exceptions", async () => {
    const setSelectedSkillId = vi.fn();
    const { result } = renderHook(() =>
      useCatalogDataCoordinator({ setSelectedSkillId }),
    );

    let loadedSkills: Awaited<ReturnType<typeof result.current.loadSkills>> = [];
    await act(async () => {
      loadedSkills = await result.current.loadSkills();
      await result.current.loadModels();
      await result.current.loadSearchConfigs();
    });

    expect(loadedSkills).toEqual([]);
    expect(result.current.skills).toEqual([]);
    expect(result.current.models).toEqual([]);
    expect(result.current.searchConfigs).toEqual([]);
    expect(result.current.hasHydratedModelConfigs).toBe(true);
    expect(result.current.hasHydratedSearchConfigs).toBe(true);
    expect(setSelectedSkillId).toHaveBeenCalledWith(null);
    expect(invoke).not.toHaveBeenCalled();
    expect(warnSpy).not.toHaveBeenCalled();
  });

  test("runtime preferences keep browser smoke quiet when Tauri internals are absent", async () => {
    const { result } = renderHook(() => useRuntimePreferencesCoordinator());

    let resolvedWorkDir = "unexpected";
    await act(async () => {
      await result.current.loadRuntimePreferences();
      resolvedWorkDir = await result.current.resolveSessionLaunchWorkDir();
    });

    expect(result.current.defaultWorkDir).toBe("");
    expect(result.current.operationPermissionMode).toBe("standard");
    expect(resolvedWorkDir).toBe("");
    expect(invoke).not.toHaveBeenCalled();
    expect(warnSpy).not.toHaveBeenCalled();
  });

  test("settings entry loaders use browser defaults without invoking Tauri", async () => {
    await expect(getRuntimePreferences()).resolves.toEqual(DEFAULT_RUNTIME_PREFERENCES);
    await expect(listModelConfigs()).resolves.toEqual([]);
    await expect(listProviderConfigs()).resolves.toEqual([]);

    expect(invoke).not.toHaveBeenCalled();
    expect(warnSpy).not.toHaveBeenCalled();
  });
});
