import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 5174,
    strictPort: true,
    watch: {
      ignored: [
        "**/.workclaw-plugin-host-fixtures/**",
        "**/.workclaw-plugin-host-cache/**",
      ],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: { target: "chrome105", minify: !process.env.TAURI_DEBUG },
});
