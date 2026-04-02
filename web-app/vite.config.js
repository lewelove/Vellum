import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'path'
import net from 'node:net'
import tls from 'node:tls'
import { Stream } from 'node:stream'

// --- BUN COMPATIBILITY SHIM ---
// Vite's proxy uses http-proxy which calls destroySoon().
// Bun does not implement this legacy Node method. 
// We patch it globally across all possible socket/stream classes.
const patch = function () { this.destroy(); };
[net.Socket, tls.TLSSocket, Stream].forEach((cls) => {
  if (cls && cls.prototype && !cls.prototype.destroySoon) {
    cls.prototype.destroySoon = patch;
  }
});

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      '$modules': path.resolve(__dirname, './src/modules'),
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
      '/ws': {
        target: 'ws://127.0.0.1:8000',
        ws: true
      }
    },
  },
})
