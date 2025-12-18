import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    // The Proxy: Your architectural bridge
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
        secure: false,
        // Optional: If your FastAPI routes don't start with /api, 
        // you can rewrite the path here. 
        // rewrite: (path) => path.replace(/^\/api/, '')
      },
    },
  },
})
