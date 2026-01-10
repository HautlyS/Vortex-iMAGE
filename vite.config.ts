/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 3 modules
 */

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";

const host = process.env.TAURI_DEV_HOST;
const isLandingDev = process.env.LANDING_DEV === 'true';
const isWebMode = process.env.VITE_WEB_MODE === 'true';

const isTauri = !!(
  process.env.TAURI_ENV_PLATFORM ||
  process.env.TAURI_ENV_ARCH ||
  process.env.TAURI_ENV_FAMILY ||
  process.env.TAURI_ENV_TARGET_TRIPLE
);

const isMobile = process.env.TAURI_ENV_PLATFORM === 'android' || 
                 process.env.TAURI_ENV_PLATFORM === 'ios';

export default defineConfig(async () => ({
  // Use relative base for GitHub Pages subdirectory deployment
  base: isWebMode ? './' : '/',
  plugins: [
    vue(),
    tailwindcss(),
  ].filter(Boolean),
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  define: {
    // Expose web mode flag to the app
    '__WEB_MODE__': JSON.stringify(isWebMode),
  },
  build: {
    target: isMobile ? 'es2020' : 'esnext',
    minify: 'esbuild',
    cssMinify: true,
    outDir: 'dist',
    emptyOutDir: true
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
  
  esbuild: {
    target: isMobile ? 'es2020' : 'esnext'
  }
}));