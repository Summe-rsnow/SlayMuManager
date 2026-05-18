import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import UnoCSS from "unocss/vite"

const host = process.env.TAURI_DEV_HOST

export default defineConfig(async () => ({
  plugins: [vue(), UnoCSS()],

  clearScreen: false,

  build: {
    rollupOptions: {
      output: {
	        chunkSizeWarningLimit: 1024,
        manualChunks(id: string) {
          if (id.includes("naive-ui")) return "naive"
          if (id.includes("lucide-vue-next")) return "icons"
          if (id.includes("node_modules/vue/") || id.includes("vue-router") || id.includes("vue-i18n")) return "vendor"
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
