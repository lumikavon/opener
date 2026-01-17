import { defineConfig } from 'vite';

export default defineConfig({
  root: './frontend',
  publicDir: 'assets',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'esnext',
    minify: 'esbuild',
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  clearScreen: false,
});
