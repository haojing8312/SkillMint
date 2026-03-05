import path from "node:path";
import { defineConfig, mergeConfig } from "vite";
import baseConfig from "../vite.config";

export default mergeConfig(
  baseConfig,
  defineConfig({
    resolve: {
      alias: {
        "./components/ChatView": path.resolve(
          __dirname,
          "./stubs/ChatView.tsx",
        ),
        "./components/employees/EmployeeHubView": path.resolve(
          __dirname,
          "./stubs/EmployeeHubView.tsx",
        ),
      },
    },
  }),
);
