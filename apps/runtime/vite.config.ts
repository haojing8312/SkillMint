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
  build: {
    target: "chrome105",
    minify: !process.env.TAURI_DEBUG,
    chunkSizeWarningLimit: 700,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes("node_modules")) {
            return undefined;
          }
          if (id.includes("react-syntax-highlighter")) {
            return "syntax-highlighter-vendor";
          }
          if (
            id.includes("react-markdown") ||
            id.includes("remark-gfm") ||
            id.includes("/remark-") ||
            id.includes("/rehype-") ||
            id.includes("/mdast-") ||
            id.includes("/micromark") ||
            id.includes("/hast-") ||
            id.includes("/unist-")
          ) {
            return "markdown-vendor";
          }
          if (id.includes("framer-motion")) {
            return "motion-vendor";
          }
          if (id.includes("@tauri-apps")) {
            return "tauri-vendor";
          }
          if (id.includes("lucide-react")) {
            return "icons-vendor";
          }
          if (
            id.includes("/react/") ||
            id.includes("/react-dom/") ||
            id.includes("/scheduler/")
          ) {
            return "react-vendor";
          }
          return undefined;
        },
      },
    },
  },
});
