import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const srcDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), 'src')

const pkg = JSON.parse(
  readFileSync(path.resolve(path.dirname(fileURLToPath(import.meta.url)), 'package.json'), 'utf-8'),
)

const tauriDevHost = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig({
  clearScreen: false,
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': srcDir,
      '@/app': path.resolve(srcDir, 'app'),
      '@/features': path.resolve(srcDir, 'features'),
      '@/components': path.resolve(srcDir, 'components'),
      '@/context': path.resolve(srcDir, 'context'),
      '@/hooks': path.resolve(srcDir, 'hooks'),
      '@/lib': path.resolve(srcDir, 'lib'),
      '@/styles': path.resolve(srcDir, 'styles'),
      '@/i18n': path.resolve(srcDir, 'i18n'),
      '@/assets': path.resolve(srcDir, 'assets'),
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  build: {
    chunkSizeWarningLimit: 700,
    rollupOptions: {
      output: {
        manualChunks(id) {
          const normalizedId = id.replace(/\\/g, '/')
          if (!normalizedId.includes('node_modules')) return undefined
          if (
            normalizedId.includes('react-syntax-highlighter') ||
            normalizedId.includes('refractor') ||
            normalizedId.includes('prismjs')
          ) {
            return 'syntax-vendor'
          }
          if (
            normalizedId.includes('react-markdown') ||
            normalizedId.includes('remark-') ||
            normalizedId.includes('hast') ||
            normalizedId.includes('mdast') ||
            normalizedId.includes('micromark') ||
            normalizedId.includes('unified')
          ) {
            return 'markdown-vendor'
          }
          return undefined
        },
      },
    },
  },
  server: {
    host: tauriDevHost || '127.0.0.1',
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:18921',
        changeOrigin: true,
      },
    },
  },
})
