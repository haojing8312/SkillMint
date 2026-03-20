import { createPluginApi } from "./api";
import { loadPluginModule, type LoadPluginModuleInput } from "./loader";
import { createPluginRegistry } from "./registry";
import { createPluginRuntime } from "./runtime";

type CliCommandStub = {
  name: string;
};

type CliProgramStub = {
  commands: CliCommandStub[];
  command: (name: string) => {
    description: (_text: string) => ReturnType<CliProgramStub["command"]>;
    option: (_flag: string, _description?: string) => ReturnType<CliProgramStub["command"]>;
    action: (_handler: (...args: unknown[]) => unknown) => ReturnType<CliProgramStub["command"]>;
  };
};

function createCliProgramStub(): CliProgramStub {
  const commands: CliCommandStub[] = [];

  return {
    commands,
    command(name: string) {
      commands.push({ name });
      const chain = {
        description() {
          return chain;
        },
        option() {
          return chain;
        },
        action() {
          return chain;
        },
      };
      return chain;
    },
  };
}

export async function executePluginRegistration(
  input: LoadPluginModuleInput & {
    shimPackageUrl?: string;
    config?: Record<string, unknown>;
  },
): Promise<{
  registry: ReturnType<typeof createPluginRegistry>;
  runtime: ReturnType<typeof createPluginRuntime>;
}> {
  const registry = createPluginRegistry();
  const runtime = createPluginRuntime({ config: input.config });
  const logger = runtime.logging.getChildLogger({ scope: "plugin-host" });
  const api = createPluginApi(registry, {
    runtime,
    logger,
    config: input.config ?? {},
    registrationMode: input.registrationMode,
    createCliContext() {
      return {
        program: createCliProgramStub(),
        config: input.config ?? {},
        logger,
      };
    },
  });

  const loaded = await loadPluginModule(input);
  const plugin = (loaded.module.default ?? loaded.module) as {
    register?: (api: ReturnType<typeof createPluginApi>) => void | Promise<void>;
  };

  if (typeof plugin.register !== "function") {
    throw new Error("plugin module must export a register(api) function");
  }

  await plugin.register(api);

  return {
    registry,
    runtime,
  };
}
