import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";
import { copyFileSync, mkdirSync } from "fs";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
const isGitHubPages = process.env.GITHUB_PAGES === 'true';

// https://vite.dev/config/
export default defineConfig(async () => ({
  base: host ? '/' : '/Vortex-iMAGE/',
  plugins: [
    vue(),
    tailwindcss(),
    isGitHubPages && {
      name: 'landing-page',
      closeBundle() {
        mkdirSync('dist', { recursive: true });
        copyFileSync('landing/index.html', 'dist/index.html');
      }
    }
  ].filter(Boolean),
  build: isGitHubPages ? {
    rollupOptions: { input: resolve(__dirname, 'landing/index.html') },
    outDir: 'dist',
    emptyOutDir: true
  } : {},
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
