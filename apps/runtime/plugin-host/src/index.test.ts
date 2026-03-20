import { describe, expect, it } from "vitest";
import { createPluginHostBootstrap, normalizeRegistrationMode } from "./index";

describe("plugin host bootstrap", () => {
  it("normalizes registration mode to a supported value", () => {
    expect(normalizeRegistrationMode("setup-runtime")).toBe("setup-runtime");
    expect(normalizeRegistrationMode("invalid-mode")).toBe("full");
  });

  it("creates a bootstrap object with resolved directories", () => {
    const bootstrap = createPluginHostBootstrap({
      pluginRootDir: "D:/plugins",
      installRootDir: "D:/plugins/install-root",
      registrationMode: "setup-only",
      runtimeBridge: {
        transport: "tauri-ipc",
        endpoint: "workclaw://plugin-host",
      },
    });

    expect(bootstrap.pluginRootDir).toBe("D:/plugins");
    expect(bootstrap.installRootDir).toBe("D:/plugins/install-root");
    expect(bootstrap.registrationMode).toBe("setup-only");
    expect(bootstrap.runtimeBridge.transport).toBe("tauri-ipc");
    expect(bootstrap.runtimeBridge.endpoint).toBe("workclaw://plugin-host");
  });
});
