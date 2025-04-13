import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [sveltekit()],

  // Fix for Tauri API imports
  build: {
    rollupOptions: {
      // Don't externalize Tauri APIs to ensure they're bundled correctly
      // external: [
      //   '@tauri-apps/api',
      //   '@tauri-apps/api/tauri',
      //   '@tauri-apps/api/dialog',
      //   '@tauri-apps/api/fs',
      //   '@tauri-apps/api/notification',
      //   '@tauri-apps/api/shell',
      //   '@tauri-apps/api/process',
      //   '@tauri-apps/api/event',
      //   '@tauri-apps/api/window',
      //   '@tauri-apps/api/path',
      //   '@tauri-apps/plugin-dialog',
      //   '@tauri-apps/plugin-fs',
      //   '@tauri-apps/plugin-notification',
      //   '@tauri-apps/plugin-shell',
      //   '@tauri-apps/plugin-process'
      // ]
    },
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // 4. Ensure proper handling of paths
  base: '',
}));
