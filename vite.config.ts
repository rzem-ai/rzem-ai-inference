import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';
import VueDevTools from 'vite-plugin-vue-devtools';
import { fileURLToPath, URL } from 'node:url';

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    // VueDevTools(), // Uncomment for debugging
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },

  // Development server configuration
  server: {
    port: 5173,
    strictPort: false,
    watch: {
      // Ignore Python backend files
      ignored: ['**/src-python/**', '**/__pycache__/**', '**/*.pyc'],
    },
  },

  // Production build configuration
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
});
