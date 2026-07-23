import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  root: "src",
  plugins: [react()],
  base: "/",
  build: {
    outDir: "../dist-web",
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    strictPort: true,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@tauri-apps/api/core": path.resolve(
        __dirname,
        "./src/web/shims/core.ts",
      ),
      "@tauri-apps/api/event": path.resolve(
        __dirname,
        "./src/web/shims/event.ts",
      ),
      "@tauri-apps/api/app": path.resolve(__dirname, "./src/web/shims/app.ts"),
      "@tauri-apps/api/window": path.resolve(
        __dirname,
        "./src/web/shims/window.ts",
      ),
      "@tauri-apps/api/path": path.resolve(
        __dirname,
        "./src/web/shims/path.ts",
      ),
      "@tauri-apps/plugin-dialog": path.resolve(
        __dirname,
        "./src/web/shims/plugin-dialog.ts",
      ),
    },
  },
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_"],
});
