/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Must match backend default (see backend/src/main.rs). Override with IRONPAD_PORT when needed.
const backendOrigin = `http://127.0.0.1:${process.env.IRONPAD_PORT ?? '37891'}`

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
  server: {
    proxy: {
      '/api': {
        target: backendOrigin,
        changeOrigin: true,
      },
      '/ws': {
        target: backendOrigin.replace('http', 'ws'),
        ws: true,
      },
    },
  },
})
