import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'path'

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      '$core': path.resolve(__dirname, './src/core'),
      '$modules': path.resolve(__dirname, './src/modules'),
      '$state': path.resolve(__dirname, './src/state'),
    }
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
        secure: false,
      },
      // Proxy Websocket traffic
      '/ws': {
        target: 'ws://127.0.0.1:8000',
        ws: true
      }
    },
  },
})
