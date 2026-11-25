import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed port.
export default defineConfig({
  clearScreen: false,
  plugins: [svelte()],
  server: { port: 1430, strictPort: true },
});
