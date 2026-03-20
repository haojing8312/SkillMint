export type PluginRegistrationMode = "full" | "setup-only" | "setup-runtime";

export type PluginHostRuntimeBridge = {
  transport: "tauri-ipc" | "stdio";
  endpoint: string;
};

export type PluginHostBootstrapInput = {
  pluginRootDir: string;
  installRootDir: string;
  registrationMode?: string;
  runtimeBridge: PluginHostRuntimeBridge;
};

export type PluginHostBootstrap = {
  pluginRootDir: string;
  installRootDir: string;
  registrationMode: PluginRegistrationMode;
  runtimeBridge: PluginHostRuntimeBridge;
};
