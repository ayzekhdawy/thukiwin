import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'path';

/**
 * Thuki Vite Configuration
 *
 * Optimized for local development with Tauri integration.
 * Multi-page build: main app + settings window.
 */
export default defineConfig(async () => ({
  plugins: [tailwindcss(), react()],
  clearScreen: false,
  build: {
    chunkSizeWarningLimit: 900,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        settings: resolve(__dirname, 'settings.html'),
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
    hmr: process.env.TAURI_DEV_HOST
      ? {
          protocol: 'ws',
          host: process.env.TAURI_DEV_HOST,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
}));