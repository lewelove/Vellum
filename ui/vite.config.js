import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'path'
import net from 'node:net'
import tls from 'node:tls'

// --- BUN COMPATIBILITY SHIM START ---
// Vite's proxy (http-proxy) uses socket.destroySoon(), which is a legacy 
// Node.js method. Bun implements node:net but omits this deprecated method.
// We patch it across all relevant Socket prototypes to ensure proxy stability.
[net.Socket, tls.TLSSocket].forEach((SocketClass) => {
  if (SocketClass && !SocketClass.prototype.destroySoon) {
    SocketClass.prototype.destroySoon = function () {
      this.destroy();
    };
  }
});
// --- BUN COMPATIBILITY SHIM END ---

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
