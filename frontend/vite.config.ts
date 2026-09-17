import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// 打包产物输出到 WinGauge/dist，与 tauri.conf.json 的 frontendDist ../dist 对齐
export default defineConfig({
  plugins: [vue()],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
})
