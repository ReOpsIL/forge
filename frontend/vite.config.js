import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  build: {
    sourcemap: true,
    minify: false,
    cssMinify: false,
    terserOptions: {
      compress: false,
      mangle: false,
      format: {
        beautify: true,
        comments: true
      }
    },
    rollupOptions: {
      output: {
        manualChunks: undefined
      }
    }
  },
  css: {
    devSourcemap: true
  }
})
