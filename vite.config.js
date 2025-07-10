import { sveltekit } from "@sveltejs/kit/vite";
import unoSvelte from "@unocss/extractor-svelte";
import unoVite from "unocss/vite";
import { defineConfig } from "vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [unoVite({ extractors: [unoSvelte()] }), await sveltekit()],

  clearScreen: false, // Prevent vite from obscuring Rust errors
  server: {
    port: 1420,
    strictPort: true, // Tauri expects a fixed port, fail if that port is not available
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: { ignored: ["**/apps/**", "**/packages/**"] },
  },
});
