/// <reference types="vitest/config" />
import { fileURLToPath, URL } from 'node:url';
import process from 'node:process';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
  // Tauri expects a fixed port and its own Rust errors on screen.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    css: false,
    coverage: {
      provider: 'v8',
      // shadcn and Dice UI registry output is vendor-like generated code;
      // application behavior is covered through its component tests.
      exclude: ['src/shared/ui/**/*.tsx'],
      // Tier 2 gate: `vitest run --coverage` fails below any of these.
      thresholds: { statements: 80, branches: 80, functions: 80, lines: 80 },
    },
  },
});
