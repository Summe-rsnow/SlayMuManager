import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import UnoCSS from "unocss/vite"
import { fileURLToPath, URL } from "node:url"

const host = process.env.TAURI_DEV_HOST

export default defineConfig(async () => ({
  plugins: [vue(), UnoCSS()],

  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  clearScreen: false,

  build: {
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    cssTarget: "chrome105",
    chunkSizeWarningLimit: 1024,
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            { name: "naive", test: /naive-ui/, priority: 20 },
            { name: "icons", test: /lucide-vue-next/, priority: 15 },
            { name: "vendor", test: /node_modules[\\/](vue|vue-router|vue-i18n)/, priority: 10 },
          ],
        },
      },
    },
  },

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
      ignored: ["**/src-tauri/**"],
    },
  },
}))
