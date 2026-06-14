import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    chunkSizeWarningLimit: 5000,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          // Vendor: Vue, Tauri API, FontAwesome — stable core deps
          if (id.includes('node_modules/vue/') || id.includes('node_modules/@vue/') ||
              id.includes('node_modules/@tauri-apps/') || id.includes('node_modules/@fortawesome/')) {
            return 'vendor'
          }
          // Charts: keep separate since data-heavy
          if (id.includes('node_modules/chart.js') || id.includes('node_modules/chartjs-plugin-zoom')) {
            return 'charts'
          }
          // Monaco: deliberately separate chunk (large)
          if (id.includes('monaco-editor')) {
            return 'monaco'
          }
        },
      },
    },
  },
})
