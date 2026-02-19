import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5174, strictPort: true },
  envPrefix: ["VITE_", "TAURI_"],
  build: { target: "chrome105", minify: !process.env.TAURI_DEBUG },
});
