import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";
import { copyFileSync, mkdirSync } from "fs";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
const isGitHubPages = process.env.GITHUB_PAGES === 'true';
const isLandingDev = process.env.LANDING_DEV === 'true';

// Tauri sets multiple env vars - check any of them
const isTauri = !!(
  process.env.TAURI_ENV_PLATFORM ||
  process.env.TAURI_ENV_ARCH ||
  process.env.TAURI_ENV_FAMILY ||
  process.env.TAURI_ENV_TARGET_TRIPLE
);

// Check if building for mobile
const isMobile = process.env.TAURI_ENV_PLATFORM === 'android' || 
                 process.env.TAURI_ENV_PLATFORM === 'ios';

// https://vite.dev/config/
export default defineConfig(async () => ({
  base: '/',
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
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  build: isGitHubPages ? {
    rollupOptions: { input: resolve(__dirname, 'landing/index.html') },
    outDir: 'dist',
    emptyOutDir: true
  } : {
    // Mobile-specific build optimizations
    target: isMobile ? 'es2020' : 'esnext',
    minify: 'esbuild',
    cssMinify: true
  },
  root: isLandingDev ? 'landing' : undefined,
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  // Ensure compatibility with mobile WebView
  esbuild: {
    target: isMobile ? 'es2020' : 'esnext'
  }
}));
