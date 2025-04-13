import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [sveltekit()],
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  build: {
    // Tauri supports es2021
    target: ['es2021', 'chrome100', 'safari13'],
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  // Prevent Vite from complaining about Tauri imports
  optimizeDeps: {
    exclude: ['@tauri-apps/api']
  },
  // Explicitly externalize Tauri imports
  ssr: {
    noExternal: ['@tauri-apps/api']
  }
});
