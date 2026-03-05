import path from "node:path";
import { defineConfig, mergeConfig } from "vite";
import baseConfig from "../vite.config";

export default mergeConfig(
  baseConfig,
  defineConfig({
    resolve: {
      alias: {
        "./components/employees/EmployeeHubView": path.resolve(
          __dirname,
          "./stubs/EmployeeHubView.tsx",
        ),
      },
    },
  }),
);
