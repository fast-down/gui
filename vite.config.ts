import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import fonts from 'unplugin-fonts/vite'
import Components from 'unplugin-vue-components/vite'
import { defineConfig } from 'vite'
import { PrimeVueResolver } from '@primevue/auto-import-resolver'
import wasm from 'vite-plugin-wasm'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    Components({
      resolvers: [PrimeVueResolver()],
      dts: 'types/components.d.ts',
    }),
    AutoImport({
      imports: [
        'vue',
        'pinia',
        '@vueuse/core',
        {
          from: '@tauri-apps/api/core',
          imports: ['invoke'],
        },
      ],
      dirs: ['src/stores', 'src/utils'],
      dts: 'types/auto-imports.d.ts',
      eslintrc: {
        enabled: true,
        filepath: 'types/.eslintrc-auto-import.json',
      },
    }),
    wasm(),
    fonts({
      fontsource: {
        families: [
          {
            name: 'Roboto',
            weights: [100, 300, 400, 500, 700, 900],
            styles: ['normal', 'italic'],
          },
        ],
      },
    }),
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
