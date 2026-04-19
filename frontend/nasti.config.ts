import { defineConfig } from '@nasti-toolchain/nasti';

export default defineConfig({
  framework: 'react',
  server: {
    port: 5174,
    proxy: {
      '/api': { target: 'http://localhost:8080', changeOrigin: true },
      '/rpc': { target: 'ws://localhost:8080', changeOrigin: true },
    },
  },
});
